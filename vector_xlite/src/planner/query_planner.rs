use crate::types::{CollectionConfig, InsertPoint, QueryPlan, SearchPoint};

pub(crate) trait QueryPlanner {
    fn plan_create_collection(&self, collection_config: CollectionConfig) -> Result<Vec<QueryPlan>, &'static str>;
    fn plan_insert_query(&self, create_point: InsertPoint) -> Result<Vec<QueryPlan>, &'static str>;
    fn plan_search_query(&self, search_point: SearchPoint) -> Result<QueryPlan, &'static str>;
}