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

    /// Checks whether a collection with the given name exists.
    ///
    /// This method verifies if a collection exists by checking for the presence of
    /// the associated tables in the database (payload table and/or virtual vector table).
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the collection exists (at least one associated table is found)
    /// * `Ok(false)` - If the collection does not exist
    /// * `Err(VecXError)` - If there's an error querying the database
    ///
    /// # Examples
    ///
    /// ```
    /// # use vector_xlite::{VectorXLite, types::CollectionConfig};
    /// # use r2d2::Pool;
    /// # use r2d2_sqlite::SqliteConnectionManager;
    /// # use vector_xlite::customizer::SqliteConnectionCustomizer;
    /// # let manager = SqliteConnectionManager::memory();
    /// # let pool = Pool::builder()
    /// #     .connection_customizer(SqliteConnectionCustomizer::new())
    /// #     .build(manager)
    /// #     .unwrap();
    /// # let vlite = VectorXLite::new(pool).unwrap();
    /// // Check if a collection exists before creating it
    /// if !vlite.collection_exists("my_collection")? {
    ///     // Create the collection if it doesn't exist
    ///     let config = CollectionConfig::builder()
    ///         .collection_name("my_collection")
    ///         .vector_dimension(128)
    ///         .build()?;
    ///     vlite.create_collection(config)?;
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn collection_exists(&self, collection_name: &str) -> Result<bool, VecXError> {
        let query_plan = self
            .query_planner
            .plan_collection_exists_query(collection_name)?;

        self.query_executor.execute_collection_exists_query(query_plan)
    }

    pub fn delete(&self, delete_point: DeletePoint) -> Result<(), VecXError> {
        let delete_query_plan = self.query_planner.plan_delete_query(delete_point)?;
        self.query_executor.execute_delete_query(delete_query_plan)
    }

    pub fn delete_collection(&self, delete_collection: DeleteCollection) -> Result<(), VecXError> {
        let delete_query_plan = self
            .query_planner
            .plan_delete_collection_query(delete_collection)?;
        self.query_executor
            .execute_delete_collection_query(delete_query_plan)
    }
}
