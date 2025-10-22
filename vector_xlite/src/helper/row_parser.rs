use rusqlite::{types::Value, Row, Result};
use std::collections::HashMap;

/// Convert a single rusqlite Value to a readable string.
pub fn get_value_as_string(row: &Row, i: usize) -> String {
    match row.get::<_, Value>(i) {
        Ok(Value::Null) => "NULL".to_string(),
        Ok(Value::Integer(v)) => v.to_string(),
        Ok(Value::Real(v)) => v.to_string(),
        Ok(Value::Text(v)) => v,
        Ok(Value::Blob(_)) => "<BLOB>".to_string(),
        Err(_) => "<ERR>".to_string(),
    }
}

/// Convert a rusqlite::Row into a HashMap<column_name, string_value>.
pub fn parse_row_to_map(row: &Row) -> Result<HashMap<String, String>> {
    let mut map: HashMap<String, String> = HashMap::new();
    for (i, col_name) in row.as_ref().column_names().iter().enumerate() {
        map.insert((*col_name).to_string(), get_value_as_string(row, i));
    }
    Ok(map)
}