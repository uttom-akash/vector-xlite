use crate::executor::{QueryExecutor,SqliteQueryExecutor};
use crate::helper::*;
use crate::planner::{QueryPlanner,SqliteQueryPlanner};
use crate::types::*;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::rc::Rc;

pub struct VectorXLite {
    query_planner: Box<dyn QueryPlanner>,
    query_executor: Box<dyn QueryExecutor>,
}

impl VectorXLite {
    pub fn new<T>(sqlite_connection: T) -> Result<VectorXLite>
    where
        T: Into<Rc<Connection>>,
    {
        let sqlite_connection = sqlite_connection.into();

        load_sqlite_vector_extension(Rc::clone(&sqlite_connection))?;

        Ok(VectorXLite {
            query_planner: SqliteQueryPlanner::new(Rc::clone(&sqlite_connection)),
            query_executor: SqliteQueryExecutor::new(sqlite_connection),
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
