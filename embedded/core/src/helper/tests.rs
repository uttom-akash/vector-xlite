#[cfg(test)]
mod tests {
    use super::super::sql::*;
    #[test]
    fn inject_rowid_with_columns_injects() {
        let sql = "insert into story(name) values ('X')";
        let out = inject_rowid(sql, 42);
        assert!(out.contains("INSERT INTO story (rowid, name) VALUES (42, 'X')") || out.to_lowercase().contains("rowid"));
    }

    #[test]
    fn replace_select_with_count_works() {
        let q = "SELECT id, name FROM story WHERE rating > 4";
        let got = replace_select_with_count(q);
        assert!(got.to_lowercase().starts_with("select count(*) from"));
    }
}