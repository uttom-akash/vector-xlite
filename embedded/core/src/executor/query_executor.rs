use crate::{error::VecXError, types::QueryPlan};

pub(crate) trait QueryExecutor: Send + Sync {
    fn execute_create_collection_query(&self, query_plans: Vec<QueryPlan>)
    -> Result<(), VecXError>;
    fn execute_insert_query(&self, query_plans: Vec<QueryPlan>) -> Result<(), VecXError>;
    fn execute_search_query(
        &self,
        query_plan: QueryPlan,
    ) -> Result<Vec<std::collections::HashMap<String, String>>, VecXError>;
}
