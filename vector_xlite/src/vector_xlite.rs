use crate::error::VecXError;
use crate::executor::{QueryExecutor, SqliteQueryExecutor};
use crate::planner::{QueryPlanner, SqliteQueryPlanner};
use crate::types::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;

pub struct VectorXLite {
    query_planner: Box<dyn QueryPlanner>,
    query_executor: Box<dyn QueryExecutor>,
}

impl VectorXLite {
    pub fn new(connection_pool: Pool<SqliteConnectionManager>) -> Result<VectorXLite, VecXError> {
        Ok(VectorXLite {
            query_planner: SqliteQueryPlanner::new(connection_pool.clone()),
            query_executor: SqliteQueryExecutor::new(connection_pool),
        })
    }
}

impl VectorXLite {
    pub fn create_collection(&self, collection_config: CollectionConfig) -> Result<(), VecXError> {
        let query_plans = self
            .query_planner
            .plan_create_collection(collection_config)?;

        self.query_executor
            .execute_create_collection_query(query_plans)
    }

    pub fn insert(&self, create_point: InsertPoint) -> Result<(), VecXError> {
        let query_plans = self.query_planner.plan_insert_query(create_point)?;

        self.query_executor.execute_insert_query(query_plans)
    }

    pub fn search(
        &self,
        search_point: SearchPoint,
    ) -> Result<Vec<HashMap<String, String>>, VecXError> {
        let query_plan = self.query_planner.plan_search_query(search_point)?;

        self.query_executor.execute_search_query(query_plan)
    }
}
