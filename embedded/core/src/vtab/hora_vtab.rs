use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

use hora::core::ann_index::ANNIndex;
use hora::core::metrics::Metric;
use hora::index::hnsw_idx::HNSWIndex;
use hora::index::hnsw_params::HNSWParams;
use rusqlite::functions::FunctionFlags;
use rusqlite::types::ValueRef;
use rusqlite::vtab::{
    update_module, Context, CreateVTab, Filters, IndexInfo, Inserts, UpdateVTab, Updates, VTab,
    VTabConnection, VTabCursor, VTabKind,
};
use rusqlite::{Connection, Error, Result};

use once_cell::sync::Lazy;

type HnswIndexType = HNSWIndex<f32, usize>;

// Global counter for unique instance IDs
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(0);

static INDICES: Lazy<RwLock<HashMap<String, Arc<RwLock<IndexData>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// Thread-local storage for current query vector (set by knn_search)
thread_local! {
    static CURRENT_QUERY_VEC: std::cell::RefCell<Option<Vec<f32>>> = const { std::cell::RefCell::new(None) };
}

struct IndexData {
    index: HnswIndexType,
    dimension: usize,
    metric: Metric,
    #[allow(dead_code)]
    max_elements: usize,
    built: bool,
    count: usize,
    // Maps hora internal index -> user rowid
    rowid_map: Vec<i64>,
    // Tracks deleted rowids (soft delete since hora doesn't support deletion)
    deleted_rowids: HashSet<i64>,
    // Stores vectors for distance computation (rowid -> vector)
    vectors: HashMap<i64, Vec<f32>>,
}

impl IndexData {
    fn new(dimension: usize, max_elements: usize, metric: Metric) -> Self {
        let mut params = HNSWParams::<f32>::default();
        params.max_item = max_elements;

        let index = HNSWIndex::new(dimension, &params);

        IndexData {
            index,
            dimension,
            metric,
            max_elements,
            built: false,
            count: 0,
            rowid_map: Vec::new(),
            deleted_rowids: HashSet::new(),
            vectors: HashMap::new(),
        }
    }

    fn compute_distance(&self, v1: &[f32], v2: &[f32]) -> f32 {
        match self.metric {
            Metric::Euclidean => {
                v1.iter()
                    .zip(v2.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    .sqrt()
            }
            Metric::CosineSimilarity => {
                let dot: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
                let mag1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let mag2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
                if mag1 == 0.0 || mag2 == 0.0 {
                    1.0
                } else {
                    1.0 - (dot / (mag1 * mag2))
                }
            }
            Metric::DotProduct => {
                -v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum::<f32>()
            }
            _ => 0.0,
        }
    }
}

#[repr(C)]
pub struct VectorLiteTab {
    base: rusqlite::vtab::sqlite3_vtab,
    table_name: String,
    index_key: String, // Unique key combining db pointer and table name
    #[allow(dead_code)]
    dimension: usize,
    #[allow(dead_code)]
    metric: Metric,
}

unsafe impl Send for VectorLiteTab {}

impl VectorLiteTab {
    fn get_index(&self) -> Option<Arc<RwLock<IndexData>>> {
        INDICES.read().ok()?.get(&self.index_key).cloned()
    }
}

// Note: We intentionally don't implement Drop to remove the index,
// because the SQLite table might still exist and be queried by other connections.
// The index is cleaned up when the table is explicitly dropped via DROP TABLE.

#[allow(dead_code)]
fn parse_metric(s: &str) -> Metric {
    match s.to_lowercase().as_str() {
        "l2" => Metric::Euclidean,
        "cosine" => Metric::CosineSimilarity,
        "ip" => Metric::DotProduct,
        _ => Metric::CosineSimilarity,
    }
}

fn parse_args(args: &[&[u8]]) -> (usize, Metric, usize) {
    let mut dimension = 3;
    let mut metric = Metric::CosineSimilarity;
    let mut max_elements = 100000;

    for arg in args.iter().skip(3) {
        let arg_str = std::str::from_utf8(arg).unwrap_or("");

        // Parse float32[N] pattern
        if arg_str.contains("float32[") {
            if let Some(start) = arg_str.find("float32[") {
                if let Some(end) = arg_str[start..].find(']') {
                    let dim_str = &arg_str[start + 8..start + end];
                    if let Ok(d) = dim_str.parse::<usize>() {
                        dimension = d;
                    }
                }
            }
            // Check for distance metric in same arg
            let lower = arg_str.to_lowercase();
            if lower.contains("cosine") {
                metric = Metric::CosineSimilarity;
            } else if lower.contains(" l2") || lower.ends_with("l2") {
                metric = Metric::Euclidean;
            } else if lower.contains(" ip") || lower.ends_with("ip") {
                metric = Metric::DotProduct;
            }
        }

        // Parse hnsw(max_elements=N) pattern
        if arg_str.contains("max_elements=") {
            if let Some(start) = arg_str.find("max_elements=") {
                let rest = &arg_str[start + 13..];
                let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(m) = num_str.parse::<usize>() {
                    max_elements = m;
                }
            }
        }
    }

    (dimension, metric, max_elements)
}

unsafe impl VTab<'_> for VectorLiteTab {
    type Aux = ();
    type Cursor = VectorLiteCursor;

    fn connect(
        _db: &mut VTabConnection,
        _aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> Result<(String, Self)> {
        let table_name = std::str::from_utf8(args[2])
            .map_err(|e| Error::ModuleError(e.to_string()))?
            .to_string();

        // Generate unique index key using instance counter
        let instance_id = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let index_key = format!("{}_{}", instance_id, table_name);

        let (dimension, metric, max_elements) = parse_args(args);

        // Create new index for this connection's table
        if let Ok(mut indices) = INDICES.write() {
            let index_data = IndexData::new(dimension, max_elements, metric);
            indices.insert(index_key.clone(), Arc::new(RwLock::new(index_data)));
        }

        let schema =
            "CREATE TABLE x(rowid INTEGER PRIMARY KEY, vector_embedding BLOB, distance REAL HIDDEN)";

        Ok((
            schema.to_string(),
            VectorLiteTab {
                base: Default::default(),
                table_name,
                index_key,
                dimension,
                metric,
            },
        ))
    }

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        // Collect constraint info first to avoid borrow issues
        let mut knn_constraint_idx: Option<usize> = None;

        for (i, constraint) in info.constraints().enumerate() {
            if constraint.is_usable() && constraint.column() == 1 {
                knn_constraint_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = knn_constraint_idx {
            let mut usage = info.constraint_usage(idx);
            usage.set_argv_index(1);
            usage.set_omit(true);

            info.set_estimated_cost(10.0);
            info.set_estimated_rows(10);
            info.set_idx_num(1);
        } else {
            let count = self
                .get_index()
                .map(|i| i.read().map(|d| d.count as i64).unwrap_or(0))
                .unwrap_or(0);
            info.set_estimated_cost(1000000.0);
            info.set_estimated_rows(count);
            info.set_idx_num(0);
        }

        Ok(())
    }

    fn open(&'_ mut self) -> Result<Self::Cursor> {
        Ok(VectorLiteCursor {
            base: Default::default(),
            index_key: self.index_key.clone(),
            results: vec![],
            current: 0,
            phantom: PhantomData,
        })
    }
}

impl CreateVTab<'_> for VectorLiteTab {
    const KIND: VTabKind = VTabKind::Default;

    fn create(
        db: &mut VTabConnection,
        aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> Result<(String, Self)> {
        // For CREATE VIRTUAL TABLE, just delegate to connect
        // connect() now always creates a fresh index with unique key
        Self::connect(db, aux, args)
    }

    fn destroy(&self) -> Result<()> {
        // Clean up the index when the table is dropped
        if let Ok(mut indices) = INDICES.write() {
            indices.remove(&self.index_key);
        }
        Ok(())
    }
}

impl UpdateVTab<'_> for VectorLiteTab {
    fn delete(&mut self, rowid: ValueRef<'_>) -> Result<()> {
        // hora doesn't support deletion, so we do soft delete by tracking deleted rowids
        let rowid_val: i64 = rowid.as_i64().unwrap_or(0);
        if let Some(index_arc) = self.get_index() {
            if let Ok(mut index_data) = index_arc.write() {
                index_data.deleted_rowids.insert(rowid_val);
            }
        }
        Ok(())
    }

    fn insert(&mut self, args: &Inserts<'_>) -> Result<i64> {
        // For schema: CREATE TABLE x(rowid PRIMARY KEY, vector_embedding, distance HIDDEN)
        // args[0] = old rowid (NULL for INSERT)
        // args[1] = new rowid
        // args[2] = rowid column value (redundant with args[1])
        // args[3] = vector_embedding column value
        // args[4] = distance column value (hidden, usually NULL)

        // args[1] can be NULL, in which case rowid is in args[2]
        let rowid: i64 = args.get(2).or_else(|_| args.get(1)).unwrap_or_else(|_| {
            self.get_index()
                .map(|i| i.read().map(|d| (d.count + 1) as i64).unwrap_or(1))
                .unwrap_or(1)
        });

        // Get vector from args[3] (vector_embedding column)
        let vector = if args.len() > 3 {
            match args.get::<Vec<u8>>(3) {
                Ok(blob) => decode_vector(&blob)?,
                Err(e) => return Err(Error::ModuleError(format!("Invalid vector data at idx 3: {:?}, args.len={}", e, args.len()))),
            }
        } else {
            return Err(Error::ModuleError(format!("Missing vector: args.len={}", args.len())));
        };

        // Add to index
        if let Some(index_arc) = self.get_index() {
            if let Ok(mut index_data) = index_arc.write() {
                // Use internal index (count) as hora's index, map to user's rowid
                let internal_idx = index_data.count;
                index_data
                    .index
                    .add(&vector, internal_idx)
                    .map_err(|e| Error::ModuleError(format!("Failed to add vector: {:?}", e)))?;

                // Store mapping from internal index to user rowid
                index_data.rowid_map.push(rowid);
                index_data.count += 1;
                // Store vector for distance computation
                index_data.vectors.insert(rowid, vector);
                // Mark as needing rebuild - we'll build lazily when searching
                index_data.built = false;
            }
        }

        Ok(rowid)
    }

    fn update(&mut self, args: &Updates<'_>) -> Result<()> {
        // args[0] = old rowid
        // args[1] = new rowid
        // args[2] = rowid column value
        // args[3] = vector_embedding column value
        // args[4] = distance column value (hidden)

        let rowid: i64 = args.get(0)?;

        if args.len() > 3 {
            let vector = match args.get::<Vec<u8>>(3) {
                Ok(blob) => decode_vector(&blob)?,
                Err(_) => return Err(Error::ModuleError("Invalid vector data".to_string())),
            };

            if let Some(index_arc) = self.get_index() {
                if let Ok(mut index_data) = index_arc.write() {
                    index_data
                        .index
                        .add(&vector, rowid as usize)
                        .map_err(|e| Error::ModuleError(format!("Failed to add vector: {:?}", e)))?;
                }
            }
        }
        Ok(())
    }
}

#[repr(C)]
pub struct VectorLiteCursor {
    base: rusqlite::vtab::sqlite3_vtab_cursor,
    index_key: String,
    results: Vec<(usize, f32)>, // (rowid, distance)
    current: usize,
    phantom: PhantomData<()>,
}

unsafe impl Send for VectorLiteCursor {}

unsafe impl VTabCursor for VectorLiteCursor {
    fn filter(
        &mut self,
        idx_num: c_int,
        _idx_str: Option<&str>,
        args: &Filters<'_>,
    ) -> Result<()> {
        self.current = 0;
        self.results.clear();

        if idx_num == 1 && !args.is_empty() {
            // KNN search mode
            let knn_param: Vec<u8> = args.get(0)?;
            let (query_vec, k) = decode_knn_param(&knn_param)?;

            if let Some(index_arc) = INDICES
                .read()
                .ok()
                .and_then(|i| i.get(&self.index_key).cloned())
            {
                // Build index lazily if needed
                if let Ok(mut index_data) = index_arc.write() {
                    if !index_data.built && index_data.count > 0 {
                        let metric = index_data.metric;
                        let _ = index_data.index.build(metric);
                        index_data.built = true;
                    }
                }
                // Now search
                if let Ok(index_data) = index_arc.read() {
                    if index_data.built && index_data.count > 0 {
                        let results = index_data.index.search(&query_vec, k);
                        // hora returns Vec<usize> of internal indices, map to rowids
                        for &internal_idx in &results {
                            if internal_idx < index_data.rowid_map.len() {
                                let rowid = index_data.rowid_map[internal_idx];
                                // Skip deleted rowids
                                if !index_data.deleted_rowids.contains(&rowid) {
                                    self.results.push((rowid as usize, 0.0));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Full table scan - return all rowids
            // Distances will be computed by column() using query vector from knn_search
            if let Some(index_arc) = INDICES
                .read()
                .ok()
                .and_then(|i| i.get(&self.index_key).cloned())
            {
                if let Ok(index_data) = index_arc.read() {
                    // Return actual rowids, not internal indices, skipping deleted ones
                    for &rowid in &index_data.rowid_map {
                        if !index_data.deleted_rowids.contains(&rowid) {
                            self.results.push((rowid as usize, 0.0));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.current += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.current >= self.results.len()
    }

    fn column(&self, ctx: &mut Context, col: c_int) -> Result<()> {
        if self.current >= self.results.len() {
            return Ok(());
        }

        let (rowid, _) = self.results[self.current];

        match col {
            0 => ctx.set_result(&(rowid as i64))?,
            1 => ctx.set_result(&rusqlite::types::Null)?, // vector_embedding
            2 => {
                // Compute actual distance using stored query vector and row vector
                let distance = CURRENT_QUERY_VEC.with(|q| {
                    if let Some(ref query_vec) = *q.borrow() {
                        if let Some(index_arc) = INDICES
                            .read()
                            .ok()
                            .and_then(|i| i.get(&self.index_key).cloned())
                        {
                            if let Ok(index_data) = index_arc.read() {
                                if let Some(row_vec) = index_data.vectors.get(&(rowid as i64)) {
                                    return index_data.compute_distance(query_vec, row_vec);
                                }
                            }
                        }
                    }
                    0.0
                });
                ctx.set_result(&(distance as f64))?
            }
            _ => ctx.set_result(&rusqlite::types::Null)?,
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        if self.current < self.results.len() {
            Ok(self.results[self.current].0 as i64)
        } else {
            Ok(0)
        }
    }
}

fn encode_vector(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4 + 4);
    bytes.extend_from_slice(&(vec.len() as u32).to_le_bytes());
    for &v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

fn decode_vector(bytes: &[u8]) -> Result<Vec<f32>> {
    if bytes.len() < 4 {
        return Err(Error::ModuleError("Invalid vector encoding".to_string()));
    }

    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let mut vec = Vec::with_capacity(len);

    for i in 0..len {
        let offset = 4 + i * 4;
        if offset + 4 > bytes.len() {
            return Err(Error::ModuleError("Invalid vector encoding".to_string()));
        }
        let v = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        vec.push(v);
    }

    Ok(vec)
}

fn encode_knn_param(vec: &[f32], k: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4 + 8);
    bytes.extend_from_slice(&(k as u32).to_le_bytes());
    bytes.extend_from_slice(&(vec.len() as u32).to_le_bytes());
    for &v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

fn decode_knn_param(bytes: &[u8]) -> Result<(Vec<f32>, usize)> {
    if bytes.len() < 8 {
        return Err(Error::ModuleError("Invalid knn_param encoding".to_string()));
    }

    let k = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let len = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;

    let mut vec = Vec::with_capacity(len);
    for i in 0..len {
        let offset = 8 + i * 4;
        if offset + 4 > bytes.len() {
            return Err(Error::ModuleError("Invalid knn_param encoding".to_string()));
        }
        let v = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        vec.push(v);
    }

    Ok((vec, k))
}

pub fn register_hora_module(conn: &Connection) -> Result<()> {
    // Register the virtual table module as "vectorlite" for compatibility
    let module = update_module::<VectorLiteTab>();
    conn.create_module("vectorlite", module, None)?;

    // Register vector_from_json function
    conn.create_scalar_function(
        "vector_from_json",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let json_str: String = ctx.get(0)?;
            let vec = parse_json_vector(&json_str)?;
            Ok(encode_vector(&vec))
        },
    )?;

    // Register knn_param function
    conn.create_scalar_function(
        "knn_param",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let vec_blob: Vec<u8> = ctx.get(0)?;
            let k: i64 = ctx.get(1)?;
            let vec = decode_vector(&vec_blob)?;

            Ok(encode_knn_param(&vec, k as usize))
        },
    )?;

    // Register knn_search as a boolean function (for WHERE clause)
    conn.create_scalar_function("knn_search", 2, FunctionFlags::SQLITE_UTF8, |ctx| {
        // Decode the knn_param to get query vector
        let knn_blob: Vec<u8> = ctx.get(1)?;
        let (query_vec, _k) = decode_knn_param(&knn_blob)?;

        // Store query vector in thread-local for distance computation
        CURRENT_QUERY_VEC.with(|q| {
            *q.borrow_mut() = Some(query_vec);
        });

        // Return true to include in results
        Ok(true)
    })?;

    Ok(())
}

fn parse_json_vector(json: &str) -> Result<Vec<f32>> {
    let trimmed = json.trim();

    // Handle array format: [1.0, 2.0, 3.0] or debug format: [1.0, 2.0, 3.0]
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    let vec: Vec<f32> = inner
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
        .collect();

    if vec.is_empty() {
        return Err(Error::ModuleError(format!(
            "Failed to parse vector from JSON: {}",
            json
        )));
    }

    Ok(vec)
}
