use crate::{executor::query_executor::QueryExecutor, types::QueryPlan};
use rusqlite::{Connection, Result};
use std::{collections::HashMap, sync::Arc};

pub(crate) struct SqliteQueryExecutor {
    conn: Arc<Connection>,
}

impl SqliteQueryExecutor {
    pub fn new(conn: Arc<Connection>) -> Box<dyn QueryExecutor> {
        Box::new(SqliteQueryExecutor { conn })
    }
}

impl QueryExecutor for SqliteQueryExecutor {
    fn execute_create_collection_query(&self, query_plans: Vec<QueryPlan>) -> Result<()> {
        query_plans.iter().try_for_each(|plan| {
            self.conn
                .execute(&plan.sql, rusqlite::params_from_iter(&plan.params))?;
            Ok(())
        })
    }

    fn execute_insert_query(&self, query_plans: Vec<QueryPlan>) -> rusqlite::Result<()> {
        query_plans.iter().try_for_each(|plan| {
            self.conn
                .execute(&plan.sql, rusqlite::params_from_iter(&plan.params))?;
            Ok(())
        })
    }

    fn execute_search_query(
        &self,
        query_plan: QueryPlan,
    ) -> rusqlite::Result<Vec<HashMap<String, String>>> {
        let mut stmt = self.conn.prepare(&query_plan.sql)?;
        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(query_plan.params),
                query_plan.post_process.unwrap(),
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
