use crate::types::QueryPlan;

pub(crate) trait QueryExecutor {
    fn execute_create_collection_query(&self, query_plans: Vec<QueryPlan>) -> rusqlite::Result<()>;
    fn execute_insert_query(&self, query_plans: Vec<QueryPlan>) -> rusqlite::Result<()>;
    fn execute_search_query(&self, query_plan: QueryPlan) -> rusqlite::Result<Vec<std::collections::HashMap<String, String>>>;
}