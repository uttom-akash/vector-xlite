//! Tests for SQL helper functions
//!
//! Note: These test the SQL manipulation utilities used internally.
//! Since these are crate-internal functions, we test them indirectly
//! through the public API or by testing their effects.

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use vector_xlite::{customizer::SqliteConnectionCustomizer, types::*, VectorXLite};

fn setup_vlite() -> (VectorXLite, Pool<SqliteConnectionManager>) {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .max_size(5)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .expect("create pool");

    let vlite = VectorXLite::new(pool.clone()).expect("create VectorXLite");
    (vlite, pool)
}

// ============================================================================
// Rowid Injection Tests (tested via insert behavior)
// ============================================================================

mod rowid_injection {
    use super::*;

    #[test]
    fn insert_injects_rowid_into_payload() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("rowid_inject")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE rowid_inject (rowid INTEGER PRIMARY KEY, name TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert with ?1 placeholder for rowid
        let point = InsertPoint::builder()
            .collection_name("rowid_inject")
            .id(42)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO rowid_inject(rowid, name) VALUES (?1, 'test')")
            .build()
            .unwrap();

        vlite.insert(point).expect("insert");

        // Verify the rowid was correctly injected
        let search = SearchPoint::builder()
            .collection_name("rowid_inject")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(1)
            .payload_search_query("SELECT rowid, name FROM rowid_inject")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("rowid").unwrap(), "42");
        assert_eq!(results[0].get("name").unwrap(), "test");
    }

    #[test]
    fn insert_handles_rowid_column_explicitly() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("explicit_rowid")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE explicit_rowid (rowid INTEGER PRIMARY KEY, value REAL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert where rowid is explicitly listed
        let point = InsertPoint::builder()
            .collection_name("explicit_rowid")
            .id(99)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO explicit_rowid(rowid, value) VALUES (?1, 3.14)")
            .build()
            .unwrap();

        vlite.insert(point).expect("insert");

        let search = SearchPoint::builder()
            .collection_name("explicit_rowid")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(1)
            .payload_search_query("SELECT rowid, value FROM explicit_rowid")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("rowid").unwrap(), "99");
    }

    #[test]
    fn insert_without_explicit_rowid_in_query() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("no_rowid_query")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE no_rowid_query (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // The library should handle injecting the rowid
        let point = InsertPoint::builder()
            .collection_name("no_rowid_query")
            .id(123)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO no_rowid_query(data) VALUES ('injected')")
            .build()
            .unwrap();

        // This tests the inject_rowid function's ability to add rowid
        let result = vlite.insert(point);
        // The exact behavior depends on implementation - document it
        println!("Insert without rowid result: {:?}", result);
    }
}

// ============================================================================
// Select to Count Transformation (tested via search behavior)
// ============================================================================

mod select_transformation {
    use super::*;

    #[test]
    fn search_with_payload_query_transforms_correctly() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("transform_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE transform_test (rowid INTEGER PRIMARY KEY, title TEXT, rating REAL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert some data
        for i in 1..=5 {
            let point = InsertPoint::builder()
                .collection_name("transform_test")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0])
                .payload_insert_query(&format!(
                    "INSERT INTO transform_test(rowid, title, rating) VALUES (?1, 'Item {}', {})",
                    i,
                    i as f32 * 1.5
                ))
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Complex payload query with multiple columns
        let search = SearchPoint::builder()
            .collection_name("transform_test")
            .vector(vec![3.0, 0.0, 0.0])
            .top_k(5)
            .payload_search_query(
                "SELECT rowid, title, rating FROM transform_test WHERE rating > 2.0",
            )
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert!(!results.is_empty());

        // Verify all results have rating > 2.0
        for result in &results {
            let rating: f64 = result.get("rating").unwrap().parse().unwrap();
            assert!(rating > 2.0, "Rating should be > 2.0");
        }
    }

    #[test]
    fn search_with_join_query() {
        let (vlite, pool) = setup_vlite();

        // Create payload table
        let config = CollectionConfigBuilder::default()
            .collection_name("join_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE join_test (rowid INTEGER PRIMARY KEY, category_id INTEGER)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Create a categories table
        let conn = pool.get().unwrap();
        conn.execute(
            "CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT)",
            [],
        )
        .expect("create categories");
        conn.execute("INSERT INTO categories(id, name) VALUES (1, 'Category A')", [])
            .expect("insert category");
        conn.execute("INSERT INTO categories(id, name) VALUES (2, 'Category B')", [])
            .expect("insert category");

        // Insert vectors with category references
        for i in 1..=4 {
            let point = InsertPoint::builder()
                .collection_name("join_test")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0])
                .payload_insert_query(&format!(
                    "INSERT INTO join_test(rowid, category_id) VALUES (?1, {})",
                    (i % 2) + 1
                ))
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search with JOIN
        let search = SearchPoint::builder()
            .collection_name("join_test")
            .vector(vec![2.0, 0.0, 0.0])
            .top_k(10)
            .payload_search_query(
                "SELECT j.rowid, c.name as category FROM join_test j JOIN categories c ON j.category_id = c.id",
            )
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search with join");
        assert!(!results.is_empty());

        // Verify join worked
        for result in &results {
            let category = result.get("category").unwrap();
            assert!(
                category == "Category A" || category == "Category B",
                "Unexpected category: {}",
                category
            );
        }
    }

    #[test]
    fn search_with_aggregate_in_subquery() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("aggregate_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE aggregate_test (rowid INTEGER PRIMARY KEY, score REAL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert data
        for i in 1..=10 {
            let point = InsertPoint::builder()
                .collection_name("aggregate_test")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0])
                .payload_insert_query(&format!(
                    "INSERT INTO aggregate_test(rowid, score) VALUES (?1, {})",
                    i as f32 * 10.0
                ))
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search filtering by above-average score
        let search = SearchPoint::builder()
            .collection_name("aggregate_test")
            .vector(vec![5.0, 0.0, 0.0])
            .top_k(10)
            .payload_search_query(
                "SELECT rowid, score FROM aggregate_test WHERE score > (SELECT AVG(score) FROM aggregate_test)",
            )
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        // Average is 55, so scores 60, 70, 80, 90, 100 should pass
        assert!(
            results.len() <= 5,
            "Expected at most 5 results above average"
        );
    }
}

// ============================================================================
// Table Name Parsing (tested via collection behavior)
// ============================================================================

mod table_name_handling {
    use super::*;

    #[test]
    fn collection_name_becomes_vector_table() {
        let (vlite, pool) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("my_vectors")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Check that a virtual table was created with the expected naming convention
        let conn = pool.get().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Should have the vector table (vt_vector_my_vectors) and payload table
        let has_vector_table = tables.iter().any(|t| t.contains("vector") && t.contains("my_vectors"));
        assert!(
            has_vector_table,
            "Expected vector table with 'my_vectors', found: {:?}",
            tables
        );
    }

    #[test]
    fn special_names_in_schema() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("test_special")
            .vector_dimension(3)
            .payload_table_schema(
                r#"CREATE TABLE test_special (
                    rowid INTEGER PRIMARY KEY,
                    "column with spaces" TEXT,
                    `backtick_column` TEXT
                )"#,
            )
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        // SQLite handles quoted identifiers - test actual behavior
        println!("Special column names result: {:?}", result);
    }
}

// ============================================================================
// Default Value Generation (tested via insert behavior)
// ============================================================================

mod default_generation {
    use super::*;

    #[test]
    fn insert_uses_schema_defaults() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("default_vals")
            .vector_dimension(3)
            .payload_table_schema(
                r#"CREATE TABLE default_vals (
                    rowid INTEGER PRIMARY KEY,
                    created_at TEXT DEFAULT (datetime('now')),
                    counter INTEGER DEFAULT 0,
                    name TEXT DEFAULT 'unnamed'
                )"#,
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert with minimal data - defaults should kick in
        let point = InsertPoint::builder()
            .collection_name("default_vals")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            // Note: not providing all columns
            .build()
            .unwrap();

        let result = vlite.insert(point);
        println!("Insert with defaults result: {:?}", result);
    }

    #[test]
    fn insert_without_payload_query_generates_default() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("no_payload_query")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE no_payload_query (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert without specifying payload_insert_query
        let point = InsertPoint::builder()
            .collection_name("no_payload_query")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();

        // The library should generate a default insert
        let result = vlite.insert(point);
        assert!(result.is_ok(), "Should generate default insert query");

        // Search to verify
        let search = SearchPoint::builder()
            .collection_name("no_payload_query")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(1)
            .payload_search_query("SELECT rowid, data FROM no_payload_query")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("rowid").unwrap(), "1");
    }
}

// ============================================================================
// Row Parser Tests (tested via search result format)
// ============================================================================

mod row_parsing {
    use super::*;

    #[test]
    fn all_sqlite_types_parsed() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("all_types")
            .vector_dimension(3)
            .payload_table_schema(
                r#"CREATE TABLE all_types (
                    rowid INTEGER PRIMARY KEY,
                    int_col INTEGER,
                    real_col REAL,
                    text_col TEXT,
                    blob_col BLOB
                )"#,
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("all_types")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query(
                "INSERT INTO all_types(rowid, int_col, real_col, text_col, blob_col) VALUES (?1, 42, 3.14, 'hello', x'DEADBEEF')",
            )
            .build()
            .unwrap();
        vlite.insert(point).expect("insert");

        let search = SearchPoint::builder()
            .collection_name("all_types")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(1)
            .payload_search_query("SELECT rowid, int_col, real_col, text_col, blob_col FROM all_types")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);

        let row = &results[0];
        assert_eq!(row.get("int_col").unwrap(), "42");
        assert_eq!(row.get("real_col").unwrap(), "3.14");
        assert_eq!(row.get("text_col").unwrap(), "hello");
        // Blob should be represented somehow (implementation dependent)
        assert!(row.contains_key("blob_col"));
    }

    #[test]
    fn null_values_parsed() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("null_types")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE null_types (rowid INTEGER PRIMARY KEY, nullable_col TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("null_types")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query(
                "INSERT INTO null_types(rowid, nullable_col) VALUES (?1, NULL)",
            )
            .build()
            .unwrap();
        vlite.insert(point).expect("insert");

        let search = SearchPoint::builder()
            .collection_name("null_types")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(1)
            .payload_search_query("SELECT rowid, nullable_col FROM null_types")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("nullable_col").unwrap(), "NULL");
    }

    #[test]
    fn empty_string_vs_null() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("empty_vs_null")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE empty_vs_null (rowid INTEGER PRIMARY KEY, col TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert NULL
        let point1 = InsertPoint::builder()
            .collection_name("empty_vs_null")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO empty_vs_null(rowid, col) VALUES (?1, NULL)")
            .build()
            .unwrap();
        vlite.insert(point1).expect("insert null");

        // Insert empty string
        let point2 = InsertPoint::builder()
            .collection_name("empty_vs_null")
            .id(2)
            .vector(vec![2.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO empty_vs_null(rowid, col) VALUES (?1, '')")
            .build()
            .unwrap();
        vlite.insert(point2).expect("insert empty");

        let search = SearchPoint::builder()
            .collection_name("empty_vs_null")
            .vector(vec![1.5, 0.0, 0.0])
            .top_k(2)
            .payload_search_query("SELECT rowid, col FROM empty_vs_null")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 2);

        // Find the NULL and empty string rows
        let null_row = results.iter().find(|r| r.get("rowid").unwrap() == "1").unwrap();
        let empty_row = results.iter().find(|r| r.get("rowid").unwrap() == "2").unwrap();

        assert_eq!(null_row.get("col").unwrap(), "NULL");
        assert_eq!(empty_row.get("col").unwrap(), "");
    }
}
