use crate::helper::sql_helpers::*;
use crate::types::{CollectionConfig, InsertPoint, QueryPlan, SearchPoint};
use rusqlite::Connection;
use std::sync::Arc;

pub struct QueryPlanner {
    pub conn: Arc<Connection>,
}

impl QueryPlanner {
    pub fn new(conn: Arc<Connection>) -> Self {
        QueryPlanner { conn }
    }

    pub fn plan_create_collection(
        &self,
        collection_config: CollectionConfig,
    ) -> Result<Vec<QueryPlan>, &'static str> {
        let mut query_plans: Vec<QueryPlan> = Vec::new();
        let virtual_table_name = get_virtual_table_name(collection_config.collection_name.as_str());

        let mut virtual_table_query = format!(
            "create virtual table {table_name} using vectorlite(vector_embedding float32[{vector_dimension}] {distance_func}, hnsw(max_elements=100000))",
            table_name = virtual_table_name,
            vector_dimension = collection_config.dimension,
            distance_func = collection_config.distance.as_str()
        );

        if let Some(index_path) = collection_config.index_file_path {
            virtual_table_query = format!("{} , {}", virtual_table_query, index_path);
        }

        query_plans.push(QueryPlan {
            sql: virtual_table_query,
            params: vec![],
            post_process: None,
        });

        if collection_config.payload_table_schema.is_some() {
            query_plans.push(QueryPlan {
                sql: collection_config.payload_table_schema.unwrap(),
                params: vec![],
                post_process: None,
            });
        }

        Ok(query_plans)
    }

    pub fn plan_insert_query(
        &self,
        create_point: InsertPoint,
    ) -> Result<Vec<QueryPlan>, &'static str> {
        let mut query_plans: Vec<QueryPlan> = Vec::new();

        let vector_json = format!("{:?}", create_point.vector);

        let virtual_table_name = get_virtual_table_name(create_point.collection_name.as_str());

        let insert_query = format!(
            "insert into {}(rowid, vector_embedding) values (?, vector_from_json(?))",
            virtual_table_name
        );

        query_plans.push(QueryPlan {
            sql: insert_query,
            params: vec![Box::new(create_point.id), Box::new(vector_json.clone())],
            post_process: None,
        });

        if create_point.payload_insert_query.is_some() {
            query_plans.push(QueryPlan {
                sql: inject_rowid(
                    create_point.payload_insert_query.as_ref().unwrap(),
                    create_point.id.unwrap(),
                ),
                params: vec![],
                post_process: None,
            });
        }

        Ok(query_plans)
    }

    pub fn plan_search_query(&self, search_point: SearchPoint) -> Result<QueryPlan, &'static str> {
        let vector_json = format!("{:?}", search_point.vector);
        let virtual_table_name = get_virtual_table_name(search_point.collection_name.as_str());

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
            .conn
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
}
