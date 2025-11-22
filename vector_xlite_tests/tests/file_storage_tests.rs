//! File Storage Tests for VectorXLite
//!
//! These tests verify that file-based SQLite databases and HNSW index files
//! work correctly with persistence across connection lifecycles.

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use vector_xlite::{customizer::SqliteConnectionCustomizer, types::*, VectorXLite};

/// Helper to create unique test file paths
fn test_paths(name: &str) -> (String, String) {
    let db_path = format!("/tmp/vxlite_test_{}.db", name);
    let idx_path = format!("/tmp/vxlite_test_{}.idx", name);
    (db_path, idx_path)
}

/// Helper to cleanup test files
fn cleanup(db_path: &str, idx_path: &str) {
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(idx_path);
}

/// Helper to create a VectorXLite instance with file storage
fn create_vlite(db_path: &str, pool_size: u32) -> (VectorXLite, Pool<SqliteConnectionManager>) {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(pool_size)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .expect("create pool");

    let vlite = VectorXLite::new(pool.clone()).expect("create VectorXLite");
    (vlite, pool)
}

// ============================================================================
// Basic File Storage Tests
// ============================================================================

mod file_persistence {
    use super::*;

    #[test]
    fn create_collection_persists_to_file() {
        let (db_path, idx_path) = test_paths("create_persist");
        cleanup(&db_path, &idx_path);

        // Create collection
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let config = CollectionConfigBuilder::default()
                .collection_name("test_collection")
                .vector_dimension(4)
                .index_file_path(&idx_path)
                .build()
                .unwrap();

            vlite.create_collection(config).expect("create collection");
        }

        // Verify files exist
        assert!(fs::metadata(&db_path).is_ok(), "DB file should exist");
        assert!(fs::metadata(&idx_path).is_ok(), "Index file should exist");

        cleanup(&db_path, &idx_path);
    }

    #[test]
    fn insert_and_search_with_file_storage() {
        let (db_path, idx_path) = test_paths("insert_search");
        cleanup(&db_path, &idx_path);

        let (vlite, _) = create_vlite(&db_path, 1);

        let config = CollectionConfigBuilder::default()
            .collection_name("search_test")
            .vector_dimension(4)
            .index_file_path(&idx_path)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert vectors
        for i in 0..20u64 {
            let point = InsertPoint::builder()
                .collection_name("search_test")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search
        let search = SearchPoint::builder()
            .collection_name("search_test")
            .vector(vec![10.0, 0.0, 0.0, 0.0])
            .top_k(20)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 20, "Should find all 20 vectors");

        cleanup(&db_path, &idx_path);
    }

    #[test]
    fn data_persists_after_connection_close() {
        let (db_path, idx_path) = test_paths("persist_close");
        cleanup(&db_path, &idx_path);

        // Phase 1: Create and insert
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let config = CollectionConfigBuilder::default()
                .collection_name("persist_test")
                .vector_dimension(4)
                .index_file_path(&idx_path)
                .build()
                .unwrap();
            vlite.create_collection(config).expect("create collection");

            for i in 0..30u64 {
                let point = InsertPoint::builder()
                    .collection_name("persist_test")
                    .id(i)
                    .vector(vec![i as f32, (i * 2) as f32, 0.0, 0.0])
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert");
            }

            // Verify before close
            let search = SearchPoint::builder()
                .collection_name("persist_test")
                .vector(vec![15.0, 30.0, 0.0, 0.0])
                .top_k(30)
                .build()
                .unwrap();
            let results = vlite.search(search).expect("search");
            assert_eq!(results.len(), 30, "Should find 30 vectors before close");
        }
        // Connection closed here

        // Phase 2: Reopen and verify data persists
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let search = SearchPoint::builder()
                .collection_name("persist_test")
                .vector(vec![15.0, 30.0, 0.0, 0.0])
                .top_k(30)
                .build()
                .unwrap();

            let results = vlite.search(search).expect("search after reopen");
            assert_eq!(results.len(), 30, "Should find all 30 vectors after reopen");
        }

        cleanup(&db_path, &idx_path);
    }

    #[test]
    fn can_add_more_data_after_reopen() {
        let (db_path, idx_path) = test_paths("add_after_reopen");
        cleanup(&db_path, &idx_path);

        // Phase 1: Create and insert initial data
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let config = CollectionConfigBuilder::default()
                .collection_name("reopen_test")
                .vector_dimension(4)
                .index_file_path(&idx_path)
                .build()
                .unwrap();
            vlite.create_collection(config).expect("create collection");

            for i in 0..25u64 {
                let point = InsertPoint::builder()
                    .collection_name("reopen_test")
                    .id(i)
                    .vector(vec![i as f32, 0.0, 0.0, 0.0])
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert");
            }
        }

        // Phase 2: Reopen and add more data
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            // Add 25 more vectors (IDs 25-49)
            for i in 25..50u64 {
                let point = InsertPoint::builder()
                    .collection_name("reopen_test")
                    .id(i)
                    .vector(vec![i as f32, 0.0, 0.0, 0.0])
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert after reopen");
            }

            // Verify all 50 vectors
            let search = SearchPoint::builder()
                .collection_name("reopen_test")
                .vector(vec![25.0, 0.0, 0.0, 0.0])
                .top_k(50)
                .build()
                .unwrap();

            let results = vlite.search(search).expect("search");
            assert_eq!(results.len(), 50, "Should find all 50 vectors");
        }

        // Phase 3: Final verification after another reopen
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let search = SearchPoint::builder()
                .collection_name("reopen_test")
                .vector(vec![25.0, 0.0, 0.0, 0.0])
                .top_k(50)
                .build()
                .unwrap();

            let results = vlite.search(search).expect("final search");
            assert_eq!(results.len(), 50, "Should find all 50 vectors after final reopen");
        }

        cleanup(&db_path, &idx_path);
    }

    #[test]
    fn index_file_grows_with_data() {
        let (db_path, idx_path) = test_paths("index_growth");
        cleanup(&db_path, &idx_path);

        // Phase 1: Create collection (index file created but may be empty)
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let config = CollectionConfigBuilder::default()
                .collection_name("growth_test")
                .vector_dimension(8)
                .index_file_path(&idx_path)
                .build()
                .unwrap();
            vlite.create_collection(config).expect("create collection");
        }
        // Connection closed - index file should exist

        let initial_size = fs::metadata(&idx_path).map(|m| m.len()).unwrap_or(0);

        // Phase 2: Insert vectors
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            for i in 0..100u64 {
                let vector: Vec<f32> = (0..8).map(|j| (i + j) as f32).collect();
                let point = InsertPoint::builder()
                    .collection_name("growth_test")
                    .id(i)
                    .vector(vector)
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert");
            }
        }
        // Connection closed - index file should be flushed

        let final_size = fs::metadata(&idx_path).map(|m| m.len()).unwrap_or(0);

        assert!(
            final_size > initial_size,
            "Index file should grow after inserts (initial: {}, final: {})",
            initial_size,
            final_size
        );

        cleanup(&db_path, &idx_path);
    }
}

// ============================================================================
// Payload Persistence Tests
// ============================================================================

mod payload_persistence {
    use super::*;

    #[test]
    fn payload_data_persists_after_reopen() {
        let (db_path, idx_path) = test_paths("payload_persist");
        cleanup(&db_path, &idx_path);

        // Phase 1: Create with payload schema and insert
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let config = CollectionConfigBuilder::default()
                .collection_name("payload_test")
                .vector_dimension(4)
                .index_file_path(&idx_path)
                .payload_table_schema(
                    "CREATE TABLE payload_test (rowid INTEGER PRIMARY KEY, name TEXT, score REAL)"
                )
                .build()
                .unwrap();
            vlite.create_collection(config).expect("create collection");

            // Insert with payload
            let point = InsertPoint::builder()
                .collection_name("payload_test")
                .id(1)
                .vector(vec![1.0, 2.0, 3.0, 4.0])
                .payload_insert_query("INSERT INTO payload_test(rowid, name, score) VALUES(?1, 'Alice', 95.5)")
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");

            let point2 = InsertPoint::builder()
                .collection_name("payload_test")
                .id(2)
                .vector(vec![5.0, 6.0, 7.0, 8.0])
                .payload_insert_query("INSERT INTO payload_test(rowid, name, score) VALUES(?1, 'Bob', 88.0)")
                .build()
                .unwrap();
            vlite.insert(point2).expect("insert");
        }

        // Phase 2: Reopen and verify payload data
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let search = SearchPoint::builder()
                .collection_name("payload_test")
                .vector(vec![1.0, 2.0, 3.0, 4.0])
                .top_k(10)
                .payload_search_query("SELECT rowid, name, score FROM payload_test")
                .build()
                .unwrap();

            let results = vlite.search(search).expect("search");
            assert_eq!(results.len(), 2, "Should find both vectors");

            // Verify payload content
            let has_alice = results.iter().any(|r| r.get("name") == Some(&"Alice".to_string()));
            let has_bob = results.iter().any(|r| r.get("name") == Some(&"Bob".to_string()));

            assert!(has_alice, "Should find Alice in results");
            assert!(has_bob, "Should find Bob in results");
        }

        cleanup(&db_path, &idx_path);
    }
}

// ============================================================================
// Multiple Collections Tests
// ============================================================================

mod multiple_collections {
    use super::*;

    #[test]
    fn multiple_collections_persist_independently() {
        let (db_path, idx_path1) = test_paths("multi_col");
        let idx_path2 = format!("{}_2", idx_path1);
        cleanup(&db_path, &idx_path1);
        let _ = fs::remove_file(&idx_path2);

        // Create two collections
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let config1 = CollectionConfigBuilder::default()
                .collection_name("collection_a")
                .vector_dimension(4)
                .index_file_path(&idx_path1)
                .build()
                .unwrap();
            vlite.create_collection(config1).expect("create collection A");

            let config2 = CollectionConfigBuilder::default()
                .collection_name("collection_b")
                .vector_dimension(4)
                .index_file_path(&idx_path2)
                .build()
                .unwrap();
            vlite.create_collection(config2).expect("create collection B");

            // Insert into collection A
            for i in 0..10u64 {
                let point = InsertPoint::builder()
                    .collection_name("collection_a")
                    .id(i)
                    .vector(vec![i as f32, 0.0, 0.0, 0.0])
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert A");
            }

            // Insert into collection B
            for i in 0..20u64 {
                let point = InsertPoint::builder()
                    .collection_name("collection_b")
                    .id(i)
                    .vector(vec![0.0, i as f32, 0.0, 0.0])
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert B");
            }
        }

        // Verify after reopen
        {
            let (vlite, _) = create_vlite(&db_path, 1);

            let search_a = SearchPoint::builder()
                .collection_name("collection_a")
                .vector(vec![5.0, 0.0, 0.0, 0.0])
                .top_k(20)
                .build()
                .unwrap();
            let results_a = vlite.search(search_a).expect("search A");
            assert_eq!(results_a.len(), 10, "Collection A should have 10 vectors");

            let search_b = SearchPoint::builder()
                .collection_name("collection_b")
                .vector(vec![0.0, 10.0, 0.0, 0.0])
                .top_k(30)
                .build()
                .unwrap();
            let results_b = vlite.search(search_b).expect("search B");
            assert_eq!(results_b.len(), 20, "Collection B should have 20 vectors");
        }

        cleanup(&db_path, &idx_path1);
        let _ = fs::remove_file(&idx_path2);
    }
}

// ============================================================================
// Distance Function Tests with File Storage
// ============================================================================

mod distance_functions_file {
    use super::*;

    #[test]
    fn cosine_distance_with_file_storage() {
        let (db_path, idx_path) = test_paths("cosine_file");
        cleanup(&db_path, &idx_path);

        let (vlite, _) = create_vlite(&db_path, 1);

        let config = CollectionConfigBuilder::default()
            .collection_name("cosine_test")
            .vector_dimension(3)
            .distance(DistanceFunction::Cosine)
            .index_file_path(&idx_path)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert normalized vectors
        let point1 = InsertPoint::builder()
            .collection_name("cosine_test")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .build()
            .unwrap();
        vlite.insert(point1).expect("insert");

        let point2 = InsertPoint::builder()
            .collection_name("cosine_test")
            .id(2)
            .vector(vec![0.0, 1.0, 0.0])
            .build()
            .unwrap();
        vlite.insert(point2).expect("insert");

        // Search for vector similar to [1,0,0]
        let search = SearchPoint::builder()
            .collection_name("cosine_test")
            .vector(vec![0.9, 0.1, 0.0])
            .top_k(2)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 2);

        // First result should be closer to [1,0,0]
        let first_id = results[0].get("rowid").unwrap();
        assert_eq!(first_id, "1", "Vector [1,0,0] should be most similar");

        cleanup(&db_path, &idx_path);
    }

    #[test]
    fn l2_distance_with_file_storage() {
        let (db_path, idx_path) = test_paths("l2_file");
        cleanup(&db_path, &idx_path);

        let (vlite, _) = create_vlite(&db_path, 1);

        let config = CollectionConfigBuilder::default()
            .collection_name("l2_test")
            .vector_dimension(2)
            .distance(DistanceFunction::L2)
            .index_file_path(&idx_path)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        let point1 = InsertPoint::builder()
            .collection_name("l2_test")
            .id(1)
            .vector(vec![0.0, 0.0])
            .build()
            .unwrap();
        vlite.insert(point1).expect("insert");

        let point2 = InsertPoint::builder()
            .collection_name("l2_test")
            .id(2)
            .vector(vec![10.0, 10.0])
            .build()
            .unwrap();
        vlite.insert(point2).expect("insert");

        // Search for vector close to origin
        let search = SearchPoint::builder()
            .collection_name("l2_test")
            .vector(vec![1.0, 1.0])
            .top_k(2)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        let first_id = results[0].get("rowid").unwrap();
        assert_eq!(first_id, "1", "Vector at origin should be closest");

        cleanup(&db_path, &idx_path);
    }
}
