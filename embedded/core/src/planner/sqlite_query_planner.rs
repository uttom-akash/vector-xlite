use crate::error::VecXError;
use crate::helper::*;
use crate::planner::query_planner::QueryPlanner;
use crate::types::{CollectionConfig, InsertPoint, QueryPlan, SearchPoint};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub(crate) struct SqliteQueryPlanner {
    conn_pool: Pool<SqliteConnectionManager>,
}

impl SqliteQueryPlanner {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Box<dyn QueryPlanner> {
        Box::new(SqliteQueryPlanner { conn_pool: pool })
    }
}

impl QueryPlanner for SqliteQueryPlanner {
    fn plan_create_collection(
        &self,
        collection_config: CollectionConfig,
    ) -> Result<Vec<QueryPlan>, VecXError> {
        let mut query_plans: Vec<QueryPlan> = Vec::new();

        if collection_config.payload_table_schema.is_some() {
            query_plans.push(QueryPlan {
                sql: collection_config.payload_table_schema.unwrap(),
                params: vec![],
                post_process: None,
            });
        }

        let virtual_table_name = get_vector_table_name(collection_config.collection_name.as_str());

        let mut virtual_table_query = format!(
            "create virtual table {table_name} using vectorlite(vector_embedding float32[{vector_dimension}] {distance_func}, hnsw(max_elements={max_elements}))",
            table_name = virtual_table_name,
            vector_dimension = collection_config.dimension,
            distance_func = collection_config.distance.as_str(),
            max_elements = collection_config.max_elements
        );

        if let Some(index_path) = collection_config.index_file_path {
            virtual_table_query = format!(
                "{} , {})",
                &virtual_table_query[0..virtual_table_query.len() - 1],
                index_path
            );
        }

        query_plans.push(QueryPlan {
            sql: virtual_table_query,
            params: vec![],
            post_process: None,
        });

        Ok(query_plans)
    }

    fn plan_insert_query(&self, create_point: InsertPoint) -> Result<Vec<QueryPlan>, VecXError> {
        let mut query_plans: Vec<QueryPlan> = Vec::new();

        let mut payload_insert_query = create_point.payload_insert_query;
        if payload_insert_query.is_none() {
            payload_insert_query = Some(
                generate_insert_with_defaults(
                    self.conn_pool.clone(),
                    create_point.collection_name.as_str(),
                )
                .unwrap(),
            );
        }

        query_plans.push(QueryPlan {
            sql: inject_rowid(
                payload_insert_query.as_ref().unwrap(),
                create_point.id.unwrap(),
            ),
            params: vec![],
            post_process: None,
        });

        let vector_json = format!("{:?}", create_point.vector);

        let virtual_table_name = get_vector_table_name(create_point.collection_name.as_str());

        let insert_query = format!(
            "insert into {}(rowid, vector_embedding) values (?, vector_from_json(?))",
            virtual_table_name
        );

        query_plans.push(QueryPlan {
            sql: insert_query,
            params: vec![Box::new(create_point.id), Box::new(vector_json.clone())],
            post_process: None,
        });

        Ok(query_plans)
    }

    fn plan_search_query(&self, search_point: SearchPoint) -> Result<QueryPlan, VecXError> {
        let vector_json = format!("{:?}", search_point.vector);
        let virtual_table_name = get_vector_table_name(search_point.collection_name.as_str());

        // --- Case 1: No payload filter ---
        if search_point.payload_search_query.is_none() {
            let sql = format!(
                "SELECT rowid, distance 
             FROM {} 
             WHERE knn_search(vector_embedding, knn_param(vector_from_json(?1), ?2)) 
             ORDER BY distance",
                virtual_table_name
            );

            return Ok(QueryPlan {
                sql,
                params: vec![Box::new(vector_json), Box::new(search_point.top_k)],
                post_process: Some(Box::new(parse_row_to_map)),
            });
        }

        let payload_query = search_point.payload_search_query.as_ref().unwrap();
        let payload_selection_count = self
            .conn_pool
            .get()?
            .query_one(
                &replace_select_with_count(search_point.payload_search_query.as_ref().unwrap()),
                (),
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count)
                },
            )
            .unwrap_or(0);

        // --- Case 2: Selective payload (< 10k rows) ---
        if payload_selection_count < 10_000 {
            let payload_query_ids = replace_select_with_row_ids(payload_query);

            let sql = format!(
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
                payload_query_ids = payload_query_ids,
                vt_table_name = virtual_table_name,
                payload_query = payload_query,
            );

            return Ok(QueryPlan {
                sql,
                params: vec![Box::new(vector_json), Box::new(search_point.top_k)],
                post_process: Some(Box::new(parse_row_to_map)),
            });
        }

        // --- Case 3: Non-selective payload (> 10k rows) ---
        let sql = format!(
            "SELECT vt.rowid, vt.distance, pt.*
         FROM (
             SELECT vt_inner.rowid, vt_inner.distance
             FROM {vt_table_name} as vt_inner
             WHERE knn_search(vt_inner.vector_embedding, knn_param(vector_from_json(?1), ?2))
         ) AS vt
         INNER JOIN ({payload_query}) AS pt
             ON vt.rowid = pt.rowid 
         ORDER BY vt.distance LIMIT ?3",
            vt_table_name = virtual_table_name,
            payload_query = payload_query,
        );

        Ok(QueryPlan {
            sql,
            params: vec![
                Box::new(vector_json),
                Box::new(10 * search_point.top_k),
                Box::new(search_point.top_k),
            ],
            post_process: Some(Box::new(parse_row_to_map)),
        })
    }

    fn plan_collection_exists_query(&self, collection_name: &str) -> Result<QueryPlan, VecXError> {
        // Check if both the payload table and the virtual vector table exist
        let virtual_table_name = get_vector_table_name(collection_name);

        // Query to check if both tables exist in sqlite_master
        let sql = format!(
            "SELECT COUNT(*) as count FROM sqlite_master WHERE type='table' AND name IN (?, ?)"
        );

        Ok(QueryPlan {
            sql,
            params: vec![
                Box::new(collection_name.to_string()),
                Box::new(virtual_table_name),
            ],
            post_process: None,
        })
    }
}
