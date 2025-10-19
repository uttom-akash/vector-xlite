use crate::helpers::extension_loaders::*;
use crate::helpers::sql_helpers::*;
use rusqlite::{Connection, Result};
use std::collections::HashMap;

// TODO : when payload table doesn't exist, create only vector table

pub trait VectorXSqlite {
    fn create_collection(&self, collection_config: CollectionConfig) -> Result<()>;

    fn insert_vector(
        &self,
        rowid: i64,
        vector: &Vec<f32>,
        payload_insert_query: &str,
    ) -> Result<()>;

    fn search_vectors(
        &self,
        query_vector: &Vec<f32>,
        top_k: i64,
        payload: &str,
    ) -> Result<Vec<HashMap<String, String>>>;
}

pub struct VectorXLite {
    pub conn: Connection,
}

pub enum DistanceFunction {
    L2,
    Cosine,
    IP,
}

pub struct CollectionConfig {
    pub vector_dimension: u16,
    pub distance: DistanceFunction,
    pub payload_table_schema: String,
    pub index_file_path: Option<String>,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            vector_dimension: 3,
            distance: DistanceFunction::Cosine,
            payload_table_schema: "".to_string(),
            index_file_path: None,
        }
    }
}

impl CollectionConfig {
    pub fn builder() -> CollectionConfigBuilder {
        CollectionConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct CollectionConfigBuilder {
    vector_dimension: Option<u16>,
    distance: Option<DistanceFunction>,
    payload_table_schema: Option<String>,
    index_file_path: Option<String>,
}

impl CollectionConfigBuilder {
    pub fn vector_dimension(mut self, dim: u16) -> Self {
        self.vector_dimension = Some(dim);
        self
    }

    pub fn distance(mut self, dist: DistanceFunction) -> Self {
        self.distance = Some(dist);
        self
    }

    pub fn payload_table_schema<S: Into<String>>(mut self, schema: S) -> Self {
        self.payload_table_schema = Some(schema.into());
        self
    }

    pub fn index_file_path<S: Into<String>>(mut self, path: S) -> Self {
        self.index_file_path = Some(path.into());
        self
    }

    pub fn build(self) -> CollectionConfig {
        let default = CollectionConfig::default();
        CollectionConfig {
            vector_dimension: self.vector_dimension.unwrap_or(default.vector_dimension),
            distance: self.distance.unwrap_or(default.distance),
            payload_table_schema: self
                .payload_table_schema
                .unwrap_or(default.payload_table_schema),
            index_file_path: self.index_file_path.or(default.index_file_path),
        }
    }
}

impl DistanceFunction {
    pub fn as_str(&self) -> &'static str {
        match self {
            DistanceFunction::L2 => "l2",
            DistanceFunction::Cosine => "cosine",
            DistanceFunction::IP => "ip",
        }
    }
}

impl VectorXLite {
    pub fn new(sqlite_connection: Connection) -> Result<Box<dyn VectorXSqlite>> {
        load_sqlite_vector_extension(&sqlite_connection)?;

        Ok(Box::new(VectorXLite {
            conn: sqlite_connection,
        }))
    }
}

impl VectorXSqlite for VectorXLite {
    fn create_collection(&self, collection_config: CollectionConfig) -> Result<()> {
        let table_name = match parse_table_name(&collection_config.payload_table_schema) {
            Some(name) => name,
            None => {
                return Err(rusqlite::Error::InvalidQuery);
            }
        };

        let mut virtual_table_query = format!(
            "create virtual table {table_name} using vectorlite(vector_embedding float32[{vector_dimension}] {distance_func}, hnsw(max_elements=100000))",
            table_name = get_virtual_table_name(table_name.as_str()),
            vector_dimension = collection_config.vector_dimension,
            distance_func = collection_config.distance.as_str()
        );

        if let Some(index_path) = collection_config.index_file_path {
            virtual_table_query = format!("{} , {}", virtual_table_query, index_path);
        }

        self.conn.execute(
            virtual_table_query.as_str(),
            (), // empty list of parameters.
        )?;

        self.conn
            .execute(&collection_config.payload_table_schema, ())?;

        Ok(())
    }

    fn insert_vector(&self, rowid: i64, vector: &Vec<f32>, payload: &str) -> Result<()> {
        let collection_name = match parse_table_name(payload) {
            Some(name) => name,
            None => {
                return Err(rusqlite::Error::InvalidQuery);
            }
        };

        let insert_query = format!(
            "insert into {}(rowid, vector_embedding) values (?, vector_from_json(?))",
            get_virtual_table_name(collection_name.as_str())
        );

        let vector_json = format!("{:?}", vector);

        self.conn
            .execute(&insert_query, (rowid, vector_json.as_str()))?;

        self.conn
            .execute(inject_rowid(payload, rowid).as_str(), ())?;

        Ok(())
    }

    fn search_vectors(
        &self,
        query_vector: &Vec<f32>,
        top_k: i64,
        payload: &str,
    ) -> Result<Vec<HashMap<String, String>>> {
        let collection_name = match parse_table_name(payload) {
            Some(name) => name,
            None => {
                return Err(rusqlite::Error::InvalidQuery);
            }
        };

        let vector_json = format!("{:?}", query_vector);

        let payload_selection_count = self
            .conn
            .query_one(&replace_select_with_count(payload), (), |row| {
                let count: i64 = row.get(0)?;
                Ok(count)
            })
            .unwrap_or(0);

        // 1. ids : attribute query in full database [when more selective]
        // 2. distance, ids : vector search in subset ids from 1
        if payload_selection_count < 10000 {
            let search_query = format!(
                "SELECT vt.rowid, vt.distance, pt.*
                    FROM (
                        SELECT vt_inner.rowid, vt_inner.distance
                        FROM {vt_table_name} as vt_inner
                        WHERE knn_search(vt_inner.vector_embedding, knn_param(vector_from_json(?1), ?2)) 
                        AND vt_inner.rowid in ({payload_query_ids})
                    ) AS vt
                INNER JOIN ({payload_query}) AS pt
                    ON vt.rowid = pt.rowid 
                    ORDER BY vt.distance LIMIT ?2",
                payload_query_ids = replace_select_with_row_ids(payload),
                vt_table_name = get_virtual_table_name(collection_name.as_str()),
                payload_query = payload,
            );

            let mut stmt = self.conn.prepare(&search_query)?;
            let rows = stmt
                .query_map((vector_json.as_str(), top_k), parse_row_to_map)?
                .collect::<Result<Vec<_>, _>>()?;

            return Ok(rows);
        }

        // 1. distance, ids : vector search in full index with over sampling
        // 2. ids : attribute query in subset ids from 1 [when less selective]
        let search_query = format!(
            "SELECT vt.rowid, vt.distance, pt.*
                FROM (
                    SELECT vt_inner.rowid, vt_inner.distance
                    FROM {collection} as vt_inner
                    WHERE knn_search(vt_inner.vector_embedding, knn_param(vector_from_json(?1), ?2))
                ) AS vt
                INNER JOIN ({payload_query}) AS pt
                    ON vt.rowid = pt.rowid 
                    ORDER BY vt.distance LIMIT ?3 ",
            collection = get_virtual_table_name(collection_name.as_str()),
            payload_query = payload,
        );

        let mut stmt = self.conn.prepare(&search_query)?;
        let rows = stmt
            .query_map((vector_json.as_str(), 10 * top_k, top_k), parse_row_to_map)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
