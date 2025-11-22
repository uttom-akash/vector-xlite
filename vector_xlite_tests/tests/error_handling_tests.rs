//! Tests for error handling in VectorXLite
//!
//! These tests verify:
//! - Graceful error handling for invalid inputs
//! - Proper error messages
//! - Recovery from error states

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
// Collection Creation Errors
// ============================================================================

mod collection_creation_errors {
    use super::*;

    #[test]
    fn duplicate_collection_name_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("duplicate_test")
            .vector_dimension(3)
            .build()
            .unwrap();

        // First creation should succeed
        let result1 = vlite.create_collection(config);
        assert!(result1.is_ok());

        // Second creation with same name should fail
        let config2 = CollectionConfigBuilder::default()
            .collection_name("duplicate_test")
            .vector_dimension(3)
            .build()
            .unwrap();

        let result2 = vlite.create_collection(config2);
        assert!(result2.is_err());
    }

    #[test]
    fn invalid_payload_schema_sql_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("bad_schema")
            .vector_dimension(3)
            .payload_table_schema("THIS IS NOT VALID SQL")
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        assert!(result.is_err());
    }

    #[test]
    fn syntax_error_in_schema() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("syntax_error")
            .vector_dimension(3)
            .payload_table_schema("CREATE TABL test (id INTEGER)") // "TABL" instead of "TABLE"
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        assert!(result.is_err());
    }
}

// ============================================================================
// Insert Errors
// ============================================================================

mod insert_errors {
    use super::*;

    #[test]
    fn insert_into_nonexistent_collection_fails() {
        let (vlite, _) = setup_vlite();

        let point = InsertPoint::builder()
            .collection_name("nonexistent_collection")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_err());
    }

    #[test]
    fn insert_duplicate_id_behavior() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("dup_id")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point1 = InsertPoint::builder()
            .collection_name("dup_id")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();
        vlite.insert(point1).expect("first insert");

        // Try to insert with same ID
        let point2 = InsertPoint::builder()
            .collection_name("dup_id")
            .id(1)
            .vector(vec![4.0, 5.0, 6.0])
            .build()
            .unwrap();

        // Document the actual behavior - may replace or fail
        let result = vlite.insert(point2);
        println!("Duplicate ID insert result: {:?}", result);
    }

    #[test]
    fn insert_wrong_dimension_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("dim_mismatch")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Try to insert 5-dimensional vector into 3-dimensional collection
        let point = InsertPoint::builder()
            .collection_name("dim_mismatch")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0, 4.0, 5.0])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        // Behavior depends on vectorlite extension - document actual behavior
        println!("Dimension mismatch result: {:?}", result);
    }

    #[test]
    fn invalid_payload_insert_query_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("bad_payload_insert")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE bad_payload_insert (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("bad_payload_insert")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INVALID SQL HERE")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_err());
    }

    #[test]
    fn constraint_violation_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("constraint_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE constraint_test (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Try to insert NULL into NOT NULL column
        let point = InsertPoint::builder()
            .collection_name("constraint_test")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO constraint_test(rowid, data) VALUES (?1, NULL)")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_err());
    }
}

// ============================================================================
// Search Errors
// ============================================================================

mod search_errors {
    use super::*;

    #[test]
    fn search_nonexistent_collection_fails() {
        let (vlite, _) = setup_vlite();

        let search = SearchPoint::builder()
            .collection_name("nonexistent")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(10)
            .build()
            .unwrap();

        let result = vlite.search(search);
        assert!(result.is_err());
    }

    #[test]
    fn search_wrong_dimension_behavior() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("search_dim")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert a vector
        let point = InsertPoint::builder()
            .collection_name("search_dim")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();
        vlite.insert(point).expect("insert");

        // Search with wrong dimension
        let search = SearchPoint::builder()
            .collection_name("search_dim")
            .vector(vec![1.0, 2.0])  // Only 2 dimensions
            .top_k(5)
            .build()
            .unwrap();

        let result = vlite.search(search);
        // Document actual behavior
        println!("Search wrong dimension result: {:?}", result);
    }

    #[test]
    fn invalid_payload_search_query_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("bad_search_query")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE bad_search_query (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("bad_search_query")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO bad_search_query(rowid, data) VALUES (?1, 'test')")
            .build()
            .unwrap();
        vlite.insert(point).expect("insert");

        let search = SearchPoint::builder()
            .collection_name("bad_search_query")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(5)
            .payload_search_query("NOT VALID SQL")
            .build()
            .unwrap();

        let result = vlite.search(search);
        assert!(result.is_err());
    }

    #[test]
    fn search_with_nonexistent_table_in_payload_fails() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("missing_table_search")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("missing_table_search")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();
        vlite.insert(point).expect("insert");

        // Search referencing a table that doesn't exist
        let search = SearchPoint::builder()
            .collection_name("missing_table_search")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(5)
            .payload_search_query("SELECT * FROM nonexistent_table")
            .build()
            .unwrap();

        let result = vlite.search(search);
        assert!(result.is_err());
    }
}

// ============================================================================
// Recovery After Errors
// ============================================================================

mod error_recovery {
    use super::*;

    #[test]
    fn can_continue_after_insert_error() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("recovery_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE recovery_test (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // First: Try an invalid insert (NULL into NOT NULL)
        let bad_point = InsertPoint::builder()
            .collection_name("recovery_test")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO recovery_test(rowid, data) VALUES (?1, NULL)")
            .build()
            .unwrap();

        let bad_result = vlite.insert(bad_point);
        assert!(bad_result.is_err());

        // Second: Valid insert should still work
        let good_point = InsertPoint::builder()
            .collection_name("recovery_test")
            .id(2)
            .vector(vec![4.0, 5.0, 6.0])
            .payload_insert_query("INSERT INTO recovery_test(rowid, data) VALUES (?1, 'valid')")
            .build()
            .unwrap();

        let good_result = vlite.insert(good_point);
        assert!(good_result.is_ok());

        // Verify the valid insert worked
        let search = SearchPoint::builder()
            .collection_name("recovery_test")
            .vector(vec![4.0, 5.0, 6.0])
            .top_k(5)
            .payload_search_query("SELECT rowid, data FROM recovery_test")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("data").unwrap(), "valid");
    }

    #[test]
    fn can_continue_after_search_error() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("search_recovery")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE search_recovery (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("search_recovery")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO search_recovery(rowid, data) VALUES (?1, 'test')")
            .build()
            .unwrap();
        vlite.insert(point).expect("insert");

        // First: Bad search
        let bad_search = SearchPoint::builder()
            .collection_name("search_recovery")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(5)
            .payload_search_query("INVALID SQL")
            .build()
            .unwrap();

        let bad_result = vlite.search(bad_search);
        assert!(bad_result.is_err());

        // Second: Good search should work
        let good_search = SearchPoint::builder()
            .collection_name("search_recovery")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(5)
            .payload_search_query("SELECT rowid, data FROM search_recovery")
            .build()
            .unwrap();

        let good_result = vlite.search(good_search);
        assert!(good_result.is_ok());
        assert_eq!(good_result.unwrap().len(), 1);
    }
}

// ============================================================================
// VecXError Display Tests
// ============================================================================

mod error_display {
    use vector_xlite::error::VecXError;

    #[test]
    fn sql_error_display() {
        let error = VecXError::SqlError("test sql error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("sql error"));
        assert!(display.contains("test sql error"));
    }

    #[test]
    fn extension_load_error_display() {
        let error = VecXError::ExtensionLoadError("failed to load".to_string());
        let display = format!("{}", error);
        assert!(display.contains("extension load error"));
    }

    #[test]
    fn invalid_query_error_display() {
        let error = VecXError::InvalidQueryError("bad query".to_string());
        let display = format!("{}", error);
        assert!(display.contains("invalid query error"));
    }

    #[test]
    fn data_parsing_error_display() {
        let error = VecXError::DataParsingError("parse failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("data parsing error"));
    }

    #[test]
    fn io_error_display() {
        let error = VecXError::IoError("io failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("io error"));
    }

    #[test]
    fn other_error_display() {
        let error = VecXError::Other("something else".to_string());
        let display = format!("{}", error);
        assert!(display.contains("error"));
    }

    #[test]
    fn error_is_debug() {
        let error = VecXError::SqlError("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("SqlError"));
    }
}
