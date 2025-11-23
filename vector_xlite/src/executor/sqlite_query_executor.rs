use crate::{error::VecXError, executor::query_executor::QueryExecutor, types::QueryPlan};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{DropBehavior, Result};
use std::collections::HashMap;

pub(crate) struct SqliteQueryExecutor {
    conn_pool: Pool<SqliteConnectionManager>,
}

impl SqliteQueryExecutor {
    pub fn new(conn_pool: Pool<SqliteConnectionManager>) -> Box<dyn QueryExecutor> {
        Box::new(SqliteQueryExecutor {
            conn_pool,
        })
    }
}

impl QueryExecutor for SqliteQueryExecutor {
    fn execute_create_collection_query(
        &self,
        query_plans: Vec<QueryPlan>,
    ) -> Result<(), VecXError> {
        let mut conn = self
                .conn_pool
                .get()?;

        let mut trx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Deferred)?;
        trx.set_drop_behavior(DropBehavior::Commit);

        query_plans.iter().try_for_each(|plan| {
            trx.execute(&plan.sql, rusqlite::params_from_iter(&plan.params))?;
            Ok(())
        })
    }

    fn execute_insert_query(&self, query_plans: Vec<QueryPlan>) -> rusqlite::Result<(), VecXError> {
        let mut conn = self
                .conn_pool
                .get()?;

        let mut trx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Deferred)?;
        trx.set_drop_behavior(DropBehavior::Commit);
        
        query_plans.iter().try_for_each(|plan| {
            trx.execute(&plan.sql, rusqlite::params_from_iter(&plan.params))?;
            Ok(())
        })
    }

    fn execute_search_query(
        &self,
        query_plan: QueryPlan,
    ) -> rusqlite::Result<Vec<HashMap<String, String>>, VecXError> {
        let conn = self.conn_pool.get()?;

        let mut stmt = conn.prepare(&query_plan.sql)?;

        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(query_plan.params),
                query_plan.post_process.unwrap(),
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
