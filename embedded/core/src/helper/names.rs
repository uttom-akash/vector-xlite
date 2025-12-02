use crate::constant::*;

pub fn get_vector_table_name(table_name: &str) -> String {
    format!("{}_{}", VECTOR_TABLE_PREFIX, table_name)
}
