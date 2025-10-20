use crate::executor::query_executor::QueryExecutor;
use crate::helper::extension_loaders::*;
use crate::planner::query_planner::QueryPlanner;
use crate::types::*;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::sync::Arc;

pub struct VectorXLite {
    pub conn: Arc<Connection>,
    pub query_planner: QueryPlanner,
    pub query_executor: QueryExecutor,
}

impl VectorXLite {
    pub fn new(sqlite_connection: Connection) -> Result<VectorXLite> {
        load_sqlite_vector_extension(&sqlite_connection)?;

        let sqlite_connection = Arc::new(sqlite_connection);

        Ok(VectorXLite {
            conn: Arc::clone(&sqlite_connection),
            query_planner: QueryPlanner::new(Arc::clone(&sqlite_connection)),
            query_executor: QueryExecutor::new(sqlite_connection),
        })
    }
}

impl VectorXLite {
    pub fn create_collection(&self, collection_config: CollectionConfig) -> Result<()> {
        let query_plans = self
            .query_planner
            .plan_create_collection(collection_config)
            .unwrap();

        self.query_executor
            .execute_create_collection_query(query_plans)
    }

    pub fn insert(&self, create_point: InsertPoint) -> Result<()> {
        let query_plans = self.query_planner.plan_insert_query(create_point).unwrap();

        self.query_executor.execute_insert_query(query_plans)
    }

    pub fn search(&self, search_point: SearchPoint) -> Result<Vec<HashMap<String, String>>> {
        let query_plan = self.query_planner.plan_search_query(search_point);

        let query_plan = match query_plan {
            Ok(plan) => plan,
            Err(_) => return Err(rusqlite::Error::InvalidQuery),
        };

        self.query_executor.execute_search_query(query_plan)
    }
}
