//! # VectorXLite Test Utilities
//!
//! A comprehensive testing framework for VectorXLite that provides:
//! - Reusable test fixtures and setup helpers
//! - Predefined collection configurations
//! - Test data generators
//! - Assertion helpers
//! - Automatic cleanup utilities
//!
//! ## Quick Start
//!
//! ```rust
//! use common::prelude::*;
//!
//! #[test]
//! fn my_test() {
//!     // Quick in-memory setup
//!     let ctx = TestContext::memory();
//!
//!     // Create a collection with one line
//!     let coll = ctx.collection("users").dimension(128).with_payload("name TEXT").create();
//!
//!     // Insert test vectors easily
//!     coll.insert_vector(1, vec![0.1; 128]).with_payload("name", "Alice").execute();
//!
//!     // Search with fluent API
//!     let results = coll.search(vec![0.1; 128]).top_k(5).execute();
//!
//!     // Use assertion helpers
//!     assert_results!(results, count = 1);
//! }
//! ```

pub mod fixtures;
pub mod assertions;
pub mod generators;
pub mod prelude;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use vector_xlite::{customizer::SqliteConnectionCustomizer, types::*, VectorXLite};

// ============================================================================
// Global Test Counter for Unique Names
// ============================================================================

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a unique test identifier
pub fn unique_id() -> usize {
    TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Generate a unique name with a prefix
pub fn unique_name(prefix: &str) -> String {
    format!("{}_{}", prefix, unique_id())
}

// ============================================================================
// Test Context - Main Entry Point
// ============================================================================

/// The main test context that manages VectorXLite instances and cleanup
pub struct TestContext {
    pub vlite: VectorXLite,
    pub pool: Pool<SqliteConnectionManager>,
    cleanup_paths: Vec<PathBuf>,
    storage_mode: StorageMode,
}

#[derive(Clone, Debug)]
pub enum StorageMode {
    Memory,
    File { db_path: PathBuf, idx_dir: PathBuf },
}

impl TestContext {
    /// Create an in-memory test context (fastest, no cleanup needed)
    pub fn memory() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(10)
            .connection_customizer(SqliteConnectionCustomizer::new())
            .build(manager)
            .expect("Failed to create connection pool");

        let vlite = VectorXLite::new(pool.clone()).expect("Failed to create VectorXLite");

        Self {
            vlite,
            pool,
            cleanup_paths: Vec::new(),
            storage_mode: StorageMode::Memory,
        }
    }

    /// Create a file-backed test context (for persistence tests)
    pub fn file() -> Self {
        let id = unique_id();
        let db_path = PathBuf::from(format!("/tmp/vxlite_test_{}.db", id));
        let idx_dir = PathBuf::from(format!("/tmp/vxlite_test_{}_idx", id));

        // Clean up any existing files
        let _ = fs::remove_file(&db_path);
        let _ = fs::remove_dir_all(&idx_dir);
        let _ = fs::create_dir_all(&idx_dir);

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder()
            .max_size(10)
            .connection_customizer(SqliteConnectionCustomizer::new())
            .build(manager)
            .expect("Failed to create connection pool");

        let vlite = VectorXLite::new(pool.clone()).expect("Failed to create VectorXLite");

        Self {
            vlite,
            pool,
            cleanup_paths: vec![db_path.clone(), idx_dir.clone()],
            storage_mode: StorageMode::File { db_path, idx_dir },
        }
    }

    /// Create a file-backed context with custom paths
    pub fn file_with_paths(db_path: &str, idx_dir: &str) -> Self {
        let db_path = PathBuf::from(db_path);
        let idx_dir = PathBuf::from(idx_dir);

        let _ = fs::remove_file(&db_path);
        let _ = fs::remove_dir_all(&idx_dir);
        let _ = fs::create_dir_all(&idx_dir);

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder()
            .max_size(10)
            .connection_customizer(SqliteConnectionCustomizer::new())
            .build(manager)
            .expect("Failed to create connection pool");

        let vlite = VectorXLite::new(pool.clone()).expect("Failed to create VectorXLite");

        Self {
            vlite,
            pool,
            cleanup_paths: vec![db_path.clone(), idx_dir.clone()],
            storage_mode: StorageMode::File { db_path, idx_dir },
        }
    }

    /// Get the index file path (for file-backed contexts)
    pub fn index_path(&self, name: &str) -> Option<String> {
        match &self.storage_mode {
            StorageMode::Memory => None,
            StorageMode::File { idx_dir, .. } => {
                Some(idx_dir.join(format!("{}.idx", name)).to_string_lossy().to_string())
            }
        }
    }

    /// Start building a collection
    pub fn collection(&self, name: &str) -> CollectionBuilder {
        CollectionBuilder::new(self, name.to_string())
    }

    /// Create a collection with default settings (3D vectors, cosine distance)
    pub fn quick_collection(&self, name: &str) -> Collection {
        self.collection(name).dimension(3).create()
    }

    /// Create a collection with payload support
    pub fn collection_with_payload(&self, name: &str, payload_columns: &str) -> Collection {
        self.collection(name)
            .dimension(3)
            .with_payload(payload_columns)
            .create()
    }

    /// Execute raw SQL on the underlying connection
    pub fn execute_sql(&self, sql: &str) -> rusqlite::Result<usize> {
        let conn = self.pool.get().expect("Get connection");
        conn.execute(sql, [])
    }

    /// Query raw SQL
    pub fn query_sql<T, F>(&self, sql: &str, f: F) -> rusqlite::Result<Vec<T>>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.pool.get().expect("Get connection");
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], f)?;
        rows.collect()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        for path in &self.cleanup_paths {
            if path.is_file() {
                let _ = fs::remove_file(path);
            } else if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            }
        }
    }
}

// ============================================================================
// Collection Builder - Fluent API for Creating Collections
// ============================================================================

pub struct CollectionBuilder<'a> {
    ctx: &'a TestContext,
    name: String,
    dimension: u16,
    distance: DistanceFunction,
    max_elements: u32,
    payload_schema: Option<String>,
}

impl<'a> CollectionBuilder<'a> {
    fn new(ctx: &'a TestContext, name: String) -> Self {
        Self {
            ctx,
            name,
            dimension: 3,
            distance: DistanceFunction::Cosine,
            max_elements: 100_000,
            payload_schema: None,
        }
    }

    /// Set vector dimension
    pub fn dimension(mut self, dim: u16) -> Self {
        self.dimension = dim;
        self
    }

    /// Set distance function
    pub fn distance(mut self, dist: DistanceFunction) -> Self {
        self.distance = dist;
        self
    }

    /// Use cosine distance
    pub fn cosine(self) -> Self {
        self.distance(DistanceFunction::Cosine)
    }

    /// Use L2 (Euclidean) distance
    pub fn l2(self) -> Self {
        self.distance(DistanceFunction::L2)
    }

    /// Use inner product distance
    pub fn inner_product(self) -> Self {
        self.distance(DistanceFunction::IP)
    }

    /// Set max elements
    pub fn max_elements(mut self, max: u32) -> Self {
        self.max_elements = max;
        self
    }

    /// Add payload support with column definitions (e.g., "name TEXT, age INTEGER")
    pub fn with_payload(mut self, columns: &str) -> Self {
        self.payload_schema = Some(format!(
            "CREATE TABLE {} (rowid INTEGER PRIMARY KEY, {})",
            self.name, columns
        ));
        self
    }

    /// Add payload with full schema SQL
    pub fn with_payload_schema(mut self, schema: &str) -> Self {
        self.payload_schema = Some(schema.to_string());
        self
    }

    /// Build and create the collection
    pub fn create(self) -> Collection<'a> {
        let mut builder = CollectionConfigBuilder::default();
        builder = builder
            .collection_name(&self.name)
            .vector_dimension(self.dimension)
            .distance(self.distance)
            .max_elements(self.max_elements);

        if let Some(schema) = &self.payload_schema {
            builder = builder.payload_table_schema(schema);
        }

        if let Some(idx_path) = self.ctx.index_path(&self.name) {
            builder = builder.index_file_path(&idx_path);
        }

        let config = builder.build().expect("Failed to build collection config");
        self.ctx
            .vlite
            .create_collection(config)
            .expect("Failed to create collection");

        Collection {
            ctx: self.ctx,
            name: self.name,
            dimension: self.dimension,
            has_payload: self.payload_schema.is_some(),
        }
    }
}

// ============================================================================
// Collection - Operations on a Created Collection
// ============================================================================

pub struct Collection<'a> {
    ctx: &'a TestContext,
    pub name: String,
    pub dimension: u16,
    pub has_payload: bool,
}

impl<'a> Collection<'a> {
    /// Start building an insert operation
    pub fn insert(&self, id: u64) -> InsertBuilder {
        InsertBuilder::new(self, id)
    }

    /// Quick insert with just a vector
    pub fn insert_vector(&self, id: u64, vector: Vec<f32>) -> &Self {
        let point = InsertPoint::builder()
            .collection_name(&self.name)
            .id(id)
            .vector(vector)
            .build()
            .expect("Build insert point");

        self.ctx.vlite.insert(point).expect("Insert vector");
        self
    }

    /// Insert multiple vectors at once
    pub fn insert_vectors(&self, vectors: Vec<(u64, Vec<f32>)>) -> &Self {
        for (id, vector) in vectors {
            self.insert_vector(id, vector);
        }
        self
    }

    /// Start building a search operation
    pub fn search(&self, vector: Vec<f32>) -> SearchBuilder {
        SearchBuilder::new(self, vector)
    }

    /// Quick search with default top_k of 10
    pub fn search_quick(&self, vector: Vec<f32>) -> Vec<std::collections::HashMap<String, String>> {
        self.search(vector).top_k(10).execute()
    }

    /// Count vectors in the collection
    pub fn count(&self) -> usize {
        self.search(vec![0.0; self.dimension as usize])
            .top_k(1_000_000)
            .execute()
            .len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

// ============================================================================
// Insert Builder - Fluent API for Inserting Vectors
// ============================================================================

pub struct InsertBuilder<'a> {
    collection: &'a Collection<'a>,
    id: u64,
    vector: Option<Vec<f32>>,
    payload_query: Option<String>,
}

impl<'a> InsertBuilder<'a> {
    fn new(collection: &'a Collection<'a>, id: u64) -> Self {
        Self {
            collection,
            id,
            vector: None,
            payload_query: None,
        }
    }

    /// Set the vector
    pub fn vector(mut self, v: Vec<f32>) -> Self {
        self.vector = Some(v);
        self
    }

    /// Set random vector with the collection's dimension
    pub fn random_vector(mut self) -> Self {
        self.vector = Some(generators::random_vector(self.collection.dimension as usize));
        self
    }

    /// Set payload insert query
    pub fn payload(mut self, query: &str) -> Self {
        self.payload_query = Some(query.to_string());
        self
    }

    /// Execute the insert
    pub fn execute(self) -> Result<(), vector_xlite::error::VecXError> {
        let vector = self.vector.unwrap_or_else(|| {
            vec![0.0; self.collection.dimension as usize]
        });

        let mut builder = InsertPoint::builder()
            .collection_name(&self.collection.name)
            .id(self.id)
            .vector(vector);

        if let Some(query) = &self.payload_query {
            builder = builder.payload_insert_query(query);
        }

        let point = builder.build().expect("Build insert point");
        self.collection.ctx.vlite.insert(point)
    }

    /// Execute and expect success
    pub fn execute_ok(self) {
        self.execute().expect("Insert should succeed");
    }

    /// Execute and expect failure
    pub fn execute_err(self) -> vector_xlite::error::VecXError {
        self.execute().expect_err("Insert should fail")
    }
}

// ============================================================================
// Search Builder - Fluent API for Searching
// ============================================================================

pub struct SearchBuilder<'a> {
    collection: &'a Collection<'a>,
    vector: Vec<f32>,
    top_k: i64,
    payload_query: Option<String>,
}

impl<'a> SearchBuilder<'a> {
    fn new(collection: &'a Collection<'a>, vector: Vec<f32>) -> Self {
        Self {
            collection,
            vector,
            top_k: 10,
            payload_query: None,
        }
    }

    /// Set top_k results
    pub fn top_k(mut self, k: i64) -> Self {
        self.top_k = k;
        self
    }

    /// Set payload search query
    pub fn payload(mut self, query: &str) -> Self {
        self.payload_query = Some(query.to_string());
        self
    }

    /// Execute the search
    pub fn execute(self) -> Vec<std::collections::HashMap<String, String>> {
        let mut builder = SearchPoint::builder()
            .collection_name(&self.collection.name)
            .vector(self.vector)
            .top_k(self.top_k);

        if let Some(query) = &self.payload_query {
            builder = builder.payload_search_query(query);
        }

        let search = builder.build().expect("Build search point");
        self.collection.ctx.vlite.search(search).expect("Search")
    }

    /// Execute and return count
    pub fn count(self) -> usize {
        self.execute().len()
    }
}

// ============================================================================
// Legacy Compatibility - For Existing Tests
// ============================================================================

/// Legacy helper - creates in-memory VectorXLite instance
/// Use `TestContext::memory()` for new tests
pub fn setup_vlite() -> (VectorXLite, Pool<SqliteConnectionManager>) {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .max_size(10)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .expect("Failed to create connection pool");

    let vlite = VectorXLite::new(pool.clone()).expect("Failed to create VectorXLite");
    (vlite, pool)
}

/// Legacy helper - creates file-backed VectorXLite instance
/// Use `TestContext::file()` for new tests
pub fn setup_vlite_with_file() -> (VectorXLite, Pool<SqliteConnectionManager>, TestPaths) {
    let id = unique_id();
    let paths = TestPaths::new(&format!("legacy_{}", id));

    let manager = SqliteConnectionManager::file(&paths.db_path);
    let pool = Pool::builder()
        .max_size(5)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .expect("create pool");

    let vlite = VectorXLite::new(pool.clone()).expect("create VectorXLite");
    (vlite, pool, paths)
}

/// Legacy test paths struct for file-based tests
pub struct TestPaths {
    pub db_path: String,
    pub idx_path: String,
}

impl TestPaths {
    pub fn new(prefix: &str) -> Self {
        let id = unique_id();
        Self {
            db_path: format!("/tmp/vxlite_{}_{}.db", prefix, id),
            idx_path: format!("/tmp/vxlite_{}_{}.idx", prefix, id),
        }
    }

    pub fn cleanup(&self) {
        let _ = fs::remove_file(&self.db_path);
        let _ = fs::remove_file(&self.idx_path);
    }
}

impl Drop for TestPaths {
    fn drop(&mut self) {
        self.cleanup();
    }
}
