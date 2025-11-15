use once_cell::sync::Lazy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use regex::Regex;

use crate::error::VecXError;

/// Compile regexes once for performance and to avoid unwraps at runtime.
static RE_WITH_COLS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)insert\s+into\s+([^\s(]+)\s*\(([^)]*)\)\s*values\s*\(([^)]*)\)").unwrap()
});
static RE_NO_COLS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^insert\s+into\s+([^\s(]+)\s*values\s*\(([^)]*)\)").unwrap());
static RE_SELECT_FROM_NONGREEDY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)SELECT\s+.*?\s+FROM").unwrap());
static RE_COLLECTION_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:table|into|from)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

/// Inject or replace `rowid` in an INSERT SQL string.
/// - If the statement lists columns and `rowid` is present, replace its corresponding value.
/// - If the statement lists columns and `rowid` is absent, inject it as the first column.
/// - If the statement has no column list, prepend the rowid value to VALUES.
///
/// This function is intentionally forgiving and returns the original SQL for unrecognized formats.
pub fn inject_rowid(sql: &str, rowid: u64) -> String {
    let rowid_str = rowid.to_string();

    if let Some(caps) = RE_WITH_COLS.captures(sql) {
        let table = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let columns = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        let values = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();

        let col_list: Vec<String> = columns.split(',').map(|s| s.trim().to_string()).collect();
        let mut val_list: Vec<String> = values.split(',').map(|s| s.trim().to_string()).collect();

        if let Some(idx) = col_list
            .iter()
            .position(|c| c.eq_ignore_ascii_case("rowid"))
        {
            // replace the corresponding value
            if idx < val_list.len() {
                val_list[idx] = rowid_str.clone();
            }
            format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                col_list.join(", "),
                val_list.join(", ")
            )
        } else {
            // inject rowid as first column & value
            format!(
                "INSERT INTO {} (rowid, {}) VALUES ({}, {})",
                table, columns, rowid_str, values
            )
        }
    } else if let Some(caps) = RE_NO_COLS.captures(sql) {
        let table = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let values = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        format!("INSERT INTO {} VALUES ({}, {})", table, rowid_str, values)
    } else {
        // Unrecognized, return original SQL
        sql.to_string()
    }
}

/// Replace the SELECT clause with a COUNT(*) selection.
pub fn replace_select_with_count(query: &str) -> String {
    RE_SELECT_FROM_NONGREEDY
        .replace(query, "SELECT count(*) FROM")
        .to_string()
}

/// Replace the SELECT clause with `SELECT rowid FROM`.
pub fn replace_select_with_row_ids(query: &str) -> String {
    RE_SELECT_FROM_NONGREEDY
        .replace(query, "SELECT rowid FROM")
        .to_string()
}

/// Try to parse a collection/table name from SQL. Returns None if not found.
pub fn parse_collection_name(sql_opt: Option<&String>) -> Option<String> {
    sql_opt.and_then(|sql| {
        RE_COLLECTION_NAME
            .captures(sql)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    })
}

#[derive(Debug)]
struct ColumnInfo {
    name: String,
    col_type: Option<String>,
    not_null: bool,
    default_value: Option<String>,
    is_pk: bool,
}

/// Create an INSERT query that includes default values for NOT NULL columns.
///
/// Rules:
/// - If column has dflt_value → use it.
/// - If NOT NULL and no dflt_value → use a synthetic safe default based on type.
/// - Else → insert NULL.
///
/// Example output:
///   INSERT INTO story(title, rating, created_at)
///   VALUES(?1, COALESCE(?2, 0.0), datetime('now'));
pub fn generate_insert_with_defaults(
    pool: Pool<SqliteConnectionManager>,
    table: &str,
) -> rusqlite::Result<String, VecXError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let col_type: Option<String> = row.get(2)?;
        let not_null: i32 = row.get(3)?;
        let default_value: Option<String> = row.get(4)?;
        let is_pk: i32 = row.get(5)?;

        Ok(ColumnInfo {
            name,
            col_type,
            not_null: not_null != 0,
            default_value,
            is_pk: is_pk != 0,
        })
    })?;

    let mut columns = Vec::<String>::new();
    let mut values = Vec::<String>::new();
    let mut placeholder_index = 1;

    for col in rows {
        let col = col?;
        columns.push(col.name.clone());

        // Case 1: Column has DEFAULT value in schema
        if let Some(def) = col.default_value.clone() {
            values.push(def); // SQLite default expressions are already valid SQL
            continue;
        }

        // Case 2: NOT NULL but no default → choose safe type-based default
        if col.not_null {
            let fallback = match col
                .col_type
                .as_deref()
                .unwrap_or("")
                .to_uppercase()
                .as_str()
            {
                "TEXT" => "''",
                "INTEGER" => "0",
                "REAL" => "0.0",
                "BLOB" => "x''",
                _ => "''", // generic fallback
            };
            values.push(fallback.to_string());
            placeholder_index += 1;
            continue;
        }

        // Case 3: Column is pk
        if col.is_pk {
            values.push(format!("?{}", placeholder_index));
            placeholder_index += 1;
            continue;
        }

        // Case 4: Column allows NULL → use placeholder wrapped with NULL default
        values.push(format!("NULL"));
    }

    let query = format!(
        "INSERT INTO {table} ({}) VALUES ({})",
        columns.join(", "),
        values.join(", ")
    );

    Ok(query)
}
