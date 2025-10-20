use std::{collections::HashMap, sync::Arc};

use rusqlite::{Connection, Result};

use crate::types::QueryPlan;

pub struct QueryExecutor {
    pub conn: Arc<Connection>,
}

impl QueryExecutor {
    pub fn new(conn: Arc<Connection>) -> Self {
        QueryExecutor { conn }
    }

    pub fn execute_create_collection_query(&self, query_plans: Vec<QueryPlan>) -> Result<()> {
        for plan in query_plans {
            self.conn
                .execute(&plan.sql, rusqlite::params_from_iter(plan.params))?;
        }

        Ok(())
    }

    pub fn execute_insert_query(&self, query_plans: Vec<QueryPlan>) -> rusqlite::Result<()> {
        for plan in query_plans {
            self.conn
                .execute(&plan.sql, rusqlite::params_from_iter(plan.params))?;
        }

        Ok(())
    }

    pub fn execute_search_query(
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
