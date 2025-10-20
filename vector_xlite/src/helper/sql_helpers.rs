use regex::Regex;
use rusqlite::{Result, Row, types::Value};
use std::collections::HashMap;

pub fn get_virtual_table_name(table_name: &str) -> String {
    format!("vt_{}", table_name)
}

pub fn inject_rowid(sql: &str, rowid: u64) -> String {
    let rowid_str = format!("{}", rowid);

    // Matches both cases: with or without explicit column list
    let re_with_cols =
        Regex::new(r"(?i)^insert\s+into\s+([^\s(]+)\s*\(([^)]*)\)\s*values\s*\(([^)]*)\)").unwrap();
    let re_no_cols = Regex::new(r"(?i)^insert\s+into\s+([^\s(]+)\s*values\s*\(([^)]*)\)").unwrap();

    if let Some(caps) = re_with_cols.captures(sql) {
        // --- Case 1: Explicit columns ---
        let table = caps.get(1).unwrap().as_str();
        let columns = caps.get(2).unwrap().as_str().trim();
        let values = caps.get(3).unwrap().as_str().trim();

        let col_list: Vec<&str> = columns.split(',').map(|s| s.trim()).collect();
        let has_rowid = col_list.iter().any(|c| c.eq_ignore_ascii_case("rowid"));

        if has_rowid {
            // Replace rowid's value
            let mut val_list: Vec<&str> = values.split(',').map(|s| s.trim()).collect();
            if let Some(idx) = col_list
                .iter()
                .position(|c| c.eq_ignore_ascii_case("rowid"))
            {
                val_list[idx] = &rowid_str;
            }
            return format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                col_list.join(", "),
                val_list.join(", ")
            );
        } else {
            // Inject rowid as first column/value
            return format!(
                "INSERT INTO {} (rowid, {}) VALUES ({}, {})",
                table, columns, rowid, values
            );
        }
    } else if let Some(caps) = re_no_cols.captures(sql) {
        // --- Case 2: No explicit columns ---
        let table = caps.get(1).unwrap().as_str();
        let values = caps.get(2).unwrap().as_str().trim();
        // Prepend rowid automatically
        return format!("INSERT INTO {} VALUES ({}, {})", table, rowid, values);
    }

    // Unrecognized format — return as is
    sql.to_string()
}

pub fn replace_select_with_count(query: &str) -> String {
    // Regex: match SELECT ... FROM (non-greedy)
    let re = Regex::new(r"(?i)^SELECT\s+.*?\s+FROM").unwrap();
    // Replace with SELECT count(*) FROM
    re.replace(query, "SELECT count(*) FROM").to_string()
}

pub fn replace_select_with_row_ids(query: &str) -> String {
    // Regex: match SELECT ... FROM (non-greedy)
    let re = Regex::new(r"(?i)^SELECT\s+.*?\s+FROM").unwrap();
    // Replace with SELECT count(*) FROM
    re.replace(query, "SELECT rowid FROM").to_string()
}

pub fn parse_collection_name(sql_opt: Option<&String>) -> Option<String> {
    
    match sql_opt {
        Some(sql) => {
            let re = Regex::new(r"(?i)\b(?:table|into|from)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
            
            re.captures(sql)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        }
        None => None,
    }
}

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

pub fn parse_row_to_map(row: &rusqlite::Row) -> Result<HashMap<String, String>> {
    let mut map: HashMap<String, String> = HashMap::new();
    for (i, col_name) in row.as_ref().column_names().iter().enumerate() {
        map.insert((*col_name).to_string(), get_value_as_string(row, i));
    }
    Ok(map)
}
