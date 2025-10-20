use std::collections::HashMap;

use rusqlite::ToSql;

pub struct QueryPlan {
    pub sql: String,

    pub params: Vec<Box<dyn ToSql>>,

    pub post_process:
        Option<Box<dyn Fn(&rusqlite::Row) -> rusqlite::Result<HashMap<String, String>>>>,
}
