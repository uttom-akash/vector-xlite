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
        Box::new(SqliteQueryExecutor { conn_pool })
    }
}

impl QueryExecutor for SqliteQueryExecutor {
    fn execute_create_collection_query(
        &self,
        query_plans: Vec<QueryPlan>,
    ) -> Result<(), VecXError> {
        let mut conn = self.conn_pool.get()?;
        let trx = conn.transaction()?;

        for plan in &query_plans {
            trx.execute(&plan.sql, rusqlite::params_from_iter(plan.params.iter()))?;
        }

        trx.commit()?;
        Ok(())
    }

    fn execute_insert_query(&self, query_plans: Vec<QueryPlan>) -> rusqlite::Result<(), VecXError> {
        let mut conn = self.conn_pool.get()?;
        let trx = conn.transaction()?;

        for plan in &query_plans {
            trx.execute(&plan.sql, rusqlite::params_from_iter(&plan.params))?;
        }

        trx.commit()?;
        Ok(())
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

    fn execute_collection_exists_query(&self, query_plan: QueryPlan) -> Result<bool, VecXError> {
        let conn = self.conn_pool.get()?;

        let count: i64 = conn.query_row(
            &query_plan.sql,
            rusqlite::params_from_iter(query_plan.params),
            |row| row.get(0),
        )?;

        // If any table exists (count >= 1), the collection is considered to exist
        Ok(count >= 1)
    }
}
