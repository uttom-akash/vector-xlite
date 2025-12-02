//! Edge case tests for VectorXLite
//!
//! These tests verify behavior at boundaries and unusual inputs:
//! - Empty collections
//! - Single-element collections
//! - Maximum dimensions
//! - Special characters in names
//! - Unicode handling
//! - Boundary values

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use vector_xlite::{customizer::SqliteConnectionCustomizer, types::*, VectorXLite};

/// Helper to create an in-memory VectorXLite instance
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
// Collection Name Edge Cases
// ============================================================================

mod collection_names {
    use super::*;

    #[test]
    fn single_character_name() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("a")
            .vector_dimension(3)
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        assert!(result.is_ok());
    }

    #[test]
    fn numeric_name() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("123")
            .vector_dimension(3)
            .build()
            .unwrap();

        // SQLite table names starting with numbers need quoting
        // This tests if the library handles it
        let result = vlite.create_collection(config);
        // Result depends on implementation - document actual behavior
        println!("Numeric name result: {:?}", result);
    }

    #[test]
    fn underscore_name() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("_underscore_start")
            .vector_dimension(3)
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        assert!(result.is_ok());
    }

    #[test]
    fn very_long_name() {
        let (vlite, _) = setup_vlite();

        // SQLite has identifier length limits
        let long_name = "a".repeat(200);
        let config = CollectionConfigBuilder::default()
            .collection_name(&long_name)
            .vector_dimension(3)
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        // Document actual behavior - may succeed or fail based on SQLite limits
        println!("Long name (200 chars) result: {:?}", result);
    }
}

// ============================================================================
// Vector Dimension Edge Cases
// ============================================================================

mod vector_dimensions {
    use super::*;

    #[test]
    fn single_dimension_vector() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("single_dim")
            .vector_dimension(1)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("single_dim")
            .id(1)
            .vector(vec![0.5])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());

        // Search with single dimension
        let search = SearchPoint::builder()
            .collection_name("single_dim")
            .vector(vec![0.5])
            .top_k(5)
            .build()
            .unwrap();

        let results = vlite.search(search);
        assert!(results.is_ok());
        assert!(!results.unwrap().is_empty());
    }

    #[test]
    fn high_dimension_vector() {
        let (vlite, _) = setup_vlite();
        let dim = 1024;

        let config = CollectionConfigBuilder::default()
            .collection_name("high_dim")
            .vector_dimension(dim as u16)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let vector: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
        let point = InsertPoint::builder()
            .collection_name("high_dim")
            .id(1)
            .vector(vector.clone())
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());

        // Search
        let search = SearchPoint::builder()
            .collection_name("high_dim")
            .vector(vector)
            .top_k(5)
            .build()
            .unwrap();

        let results = vlite.search(search);
        assert!(results.is_ok());
    }
}

// ============================================================================
// Search Result Edge Cases
// ============================================================================

mod search_results {
    use super::*;

    #[test]
    fn search_empty_collection() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("empty_collection")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let search = SearchPoint::builder()
            .collection_name("empty_collection")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(10)
            .build()
            .unwrap();

        let results = vlite.search(search);
        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }

    #[test]
    fn search_top_k_larger_than_collection() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("small_collection")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Insert only 2 vectors
        for id in 1..=2 {
            let point = InsertPoint::builder()
                .collection_name("small_collection")
                .id(id)
                .vector(vec![id as f32, id as f32, id as f32])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search for 100 results when only 2 exist
        let search = SearchPoint::builder()
            .collection_name("small_collection")
            .vector(vec![1.5, 1.5, 1.5])
            .top_k(100)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_top_k_equals_one() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("topk_one")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Insert 5 vectors
        for id in 1..=5 {
            let point = InsertPoint::builder()
                .collection_name("topk_one")
                .id(id)
                .vector(vec![id as f32, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let search = SearchPoint::builder()
            .collection_name("topk_one")
            .vector(vec![3.0, 0.0, 0.0])
            .top_k(1)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
    }
}

// ============================================================================
// Vector Value Edge Cases
// ============================================================================

mod vector_values {
    use super::*;

    #[test]
    fn all_zero_vector() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("zero_vec")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("zero_vec")
            .id(1)
            .vector(vec![0.0, 0.0, 0.0])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());
    }

    #[test]
    fn negative_vector_values() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("negative_vec")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("negative_vec")
            .id(1)
            .vector(vec![-1.0, -2.0, -3.0])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());

        let search = SearchPoint::builder()
            .collection_name("negative_vec")
            .vector(vec![-1.0, -2.0, -3.0])
            .top_k(1)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn very_small_vector_values() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("small_vals")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("small_vals")
            .id(1)
            .vector(vec![1e-38, 1e-38, 1e-38])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());
    }

    #[test]
    fn very_large_vector_values() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("large_vals")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("large_vals")
            .id(1)
            .vector(vec![1e38, 1e38, 1e38])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());
    }

    #[test]
    fn mixed_sign_vector() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("mixed_vec")
            .vector_dimension(4)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("mixed_vec")
            .id(1)
            .vector(vec![-1.0, 1.0, -0.5, 0.5])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());
    }
}

// ============================================================================
// ID Edge Cases
// ============================================================================

mod id_handling {
    use super::*;

    #[test]
    fn insert_with_id_zero() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("id_zero")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("id_zero")
            .id(0)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();

        // SQLite rowid 0 might have special handling
        let result = vlite.insert(point);
        println!("Insert with id=0 result: {:?}", result);
    }

    #[test]
    fn insert_with_large_id() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("large_id")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("large_id")
            .id(u64::MAX / 2) // Large but not max to avoid SQLite issues
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();

        let result = vlite.insert(point);
        println!("Insert with large id result: {:?}", result);
    }

    #[test]
    fn sequential_ids() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("sequential")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        for id in 1..=10u64 {
            let point = InsertPoint::builder()
                .collection_name("sequential")
                .id(id)
                .vector(vec![id as f32, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let search = SearchPoint::builder()
            .collection_name("sequential")
            .vector(vec![5.0, 0.0, 0.0])
            .top_k(10)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn non_sequential_ids() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("non_sequential")
            .vector_dimension(3)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let ids = vec![100, 5, 999, 42, 1];
        for id in ids {
            let point = InsertPoint::builder()
                .collection_name("non_sequential")
                .id(id)
                .vector(vec![id as f32, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let search = SearchPoint::builder()
            .collection_name("non_sequential")
            .vector(vec![50.0, 0.0, 0.0])
            .top_k(5)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 5);
    }
}

// ============================================================================
// Payload Edge Cases
// ============================================================================

mod payload_handling {
    use super::*;

    #[test]
    fn empty_payload_query() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("empty_payload")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE empty_payload (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("empty_payload")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO empty_payload(rowid, data) VALUES (?1, '')")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());
    }

    #[test]
    fn null_in_payload() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("null_payload")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE null_payload (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let point = InsertPoint::builder()
            .collection_name("null_payload")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO null_payload(rowid, data) VALUES (?1, NULL)")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());

        // Search and verify NULL handling
        let search = SearchPoint::builder()
            .collection_name("null_payload")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(1)
            .payload_search_query("SELECT rowid, data FROM null_payload")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("data").unwrap(), "NULL");
    }

    #[test]
    fn special_characters_in_payload() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("special_chars")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE special_chars (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Insert with special characters (properly escaped)
        let point = InsertPoint::builder()
            .collection_name("special_chars")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query(
                "INSERT INTO special_chars(rowid, data) VALUES (?1, 'test''s value')",
            )
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok());
    }
}

// ============================================================================
// Multiple Collections
// ============================================================================

mod multiple_collections {
    use super::*;

    #[test]
    fn create_multiple_collections() {
        let (vlite, _) = setup_vlite();

        for i in 1..=5 {
            let config = CollectionConfigBuilder::default()
                .collection_name(format!("collection_{}", i))
                .vector_dimension(3)
                .build()
                .unwrap();

            vlite.create_collection(config).expect("create collection");
        }

        // Insert and search in each
        for i in 1..=5 {
            let name = format!("collection_{}", i);

            let point = InsertPoint::builder()
                .collection_name(&name)
                .id(1)
                .vector(vec![i as f32, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");

            let search = SearchPoint::builder()
                .collection_name(&name)
                .vector(vec![i as f32, 0.0, 0.0])
                .top_k(1)
                .build()
                .unwrap();

            let results = vlite.search(search).expect("search");
            assert_eq!(results.len(), 1);
        }
    }

    #[test]
    fn collections_with_different_dimensions() {
        let (vlite, _) = setup_vlite();

        let dimensions = [2, 4, 8, 16, 32];

        for dim in dimensions {
            let config = CollectionConfigBuilder::default()
                .collection_name(format!("dim_{}", dim))
                .vector_dimension(dim)
                .build()
                .unwrap();

            vlite.create_collection(config).expect("create collection");

            let vector: Vec<f32> = (0..dim).map(|i| i as f32).collect();
            let point = InsertPoint::builder()
                .collection_name(format!("dim_{}", dim))
                .id(1)
                .vector(vector.clone())
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");

            let search = SearchPoint::builder()
                .collection_name(format!("dim_{}", dim))
                .vector(vector)
                .top_k(1)
                .build()
                .unwrap();

            let results = vlite.search(search).expect("search");
            assert_eq!(results.len(), 1);
        }
    }
}
