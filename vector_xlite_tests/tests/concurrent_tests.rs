//! Concurrency tests for VectorXLite
//!
//! These tests verify thread-safety and concurrent access patterns:
//! - Parallel inserts
//! - Parallel searches
//! - Mixed read/write workloads
//! - Connection pool behavior
//!
//! NOTE: These tests use file-based SQLite storage which is more realistic
//! for production use cases and provides better durability than in-memory databases.

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::thread;
use vector_xlite::{customizer::SqliteConnectionCustomizer, types::*, VectorXLite};

/// Global counter for generating unique database names across tests
static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Test file paths for a given test ID
struct TestPaths {
    db_path: String,
    idx_path: String,
}

impl TestPaths {
    fn new(prefix: &str) -> Self {
        let id = DB_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
        Self {
            db_path: format!("/tmp/vxlite_concurrent_{}_{}.db", prefix, id),
            idx_path: format!("/tmp/vxlite_concurrent_{}_{}.idx", prefix, id),
        }
    }

    fn cleanup(&self) {
        let _ = fs::remove_file(&self.db_path);
        let _ = fs::remove_file(&self.idx_path);
    }
}

impl Drop for TestPaths {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Creates a file-based SQLite database with connection pooling and HNSW index file.
///
/// This setup uses:
/// - File-based SQLite database for durability
/// - File-based HNSW index for vector persistence
/// - Configurable connection pool size for concurrency testing
fn setup_vlite_with_pool_size(size: u32) -> (Arc<VectorXLite>, Pool<SqliteConnectionManager>, TestPaths) {
    let paths = TestPaths::new("test");
    paths.cleanup(); // Ensure clean state

    let manager = SqliteConnectionManager::file(&paths.db_path);
    let pool = Pool::builder()
        .max_size(size)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .expect("create pool");

    let vlite = Arc::new(VectorXLite::new(pool.clone()).expect("create VectorXLite"));
    (vlite, pool, paths)
}

/// Creates a collection with file-based HNSW index
fn create_collection_with_index(vlite: &VectorXLite, name: &str, dimension: u16, idx_path: &str) {
    let config = CollectionConfigBuilder::default()
        .collection_name(name)
        .vector_dimension(dimension)
        .index_file_path(idx_path)
        .build()
        .unwrap();
    vlite.create_collection(config).expect("create collection");
}

// ============================================================================
// Parallel Insert Tests
// ============================================================================

mod parallel_inserts {
    use super::*;

    #[test]
    #[ignore]
    fn parallel_inserts_same_collection() {
        let (vlite, _, paths) = setup_vlite_with_pool_size(10);

        create_collection_with_index(&vlite, "parallel_insert", 8, &paths.idx_path);

        let num_threads = 4;
        let inserts_per_thread = 25;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let vlite_clone = Arc::clone(&vlite);
                thread::spawn(move || {
                    for i in 0..inserts_per_thread {
                        let id = (thread_id * 1000 + i) as u64;
                        let vector: Vec<f32> = (0..8).map(|j| (id + j) as f32 / 100.0).collect();

                        let point = InsertPoint::builder()
                            .collection_name("parallel_insert")
                            .id(id)
                            .vector(vector)
                            .build()
                            .unwrap();

                        vlite_clone.insert(point).expect("insert");
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // Verify all inserts succeeded
        let search = SearchPoint::builder()
            .collection_name("parallel_insert")
            .vector(vec![0.5; 8])
            .top_k((num_threads * inserts_per_thread) as i64)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(
            results.len(),
            (num_threads * inserts_per_thread) as usize,
            "Expected all parallel inserts to succeed"
        );
    }

    #[test]
    #[ignore]
    fn parallel_inserts_multiple_collections() {
        let (vlite, _, _paths) = setup_vlite_with_pool_size(10);

        let num_collections = 4;
        let inserts_per_collection = 20;

        // Create collections first with separate index files
        let idx_paths: Vec<String> = (0..num_collections)
            .map(|i| format!("/tmp/vxlite_parallel_col_{}.idx", i))
            .collect();

        for i in 0..num_collections {
            // Clean up any existing index files
            let _ = fs::remove_file(&idx_paths[i]);
            create_collection_with_index(&vlite, &format!("parallel_col_{}", i), 4, &idx_paths[i]);
        }

        let handles: Vec<_> = (0..num_collections)
            .map(|col_id| {
                let vlite_clone = Arc::clone(&vlite);
                thread::spawn(move || {
                    let collection_name = format!("parallel_col_{}", col_id);
                    for i in 0..inserts_per_collection {
                        let id = i as u64;
                        let vector = vec![col_id as f32, i as f32, 0.0, 0.0];

                        let point = InsertPoint::builder()
                            .collection_name(&collection_name)
                            .id(id)
                            .vector(vector)
                            .build()
                            .unwrap();

                        vlite_clone.insert(point).expect("insert");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // Verify each collection has correct number of entries
        for i in 0..num_collections {
            let search = SearchPoint::builder()
                .collection_name(format!("parallel_col_{}", i))
                .vector(vec![i as f32, 10.0, 0.0, 0.0])
                .top_k(100)
                .build()
                .unwrap();

            let results = vlite.search(search).expect("search");
            assert_eq!(
                results.len(),
                inserts_per_collection,
                "Expected {} inserts in collection {}",
                inserts_per_collection,
                i
            );
        }

        // Cleanup index files
        for path in &idx_paths {
            let _ = fs::remove_file(path);
        }
    }
}

// ============================================================================
// Parallel Search Tests
// ============================================================================

mod parallel_searches {
    use super::*;

    #[test]
    #[ignore]
    fn parallel_searches_same_collection() {
        let (vlite, _, paths) = setup_vlite_with_pool_size(10);

        create_collection_with_index(&vlite, "parallel_search", 4, &paths.idx_path);

        // Insert some data
        for i in 0..100 {
            let point = InsertPoint::builder()
                .collection_name("parallel_search")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let num_threads = 8;
        let searches_per_thread = 10;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let vlite_clone = Arc::clone(&vlite);
                thread::spawn(move || {
                    let mut results_count = 0;
                    for i in 0..searches_per_thread {
                        let query_val = (thread_id * searches_per_thread + i) as f32;
                        let search = SearchPoint::builder()
                            .collection_name("parallel_search")
                            .vector(vec![query_val, 0.0, 0.0, 0.0])
                            .top_k(5)
                            .build()
                            .unwrap();

                        let results = vlite_clone.search(search).expect("search");
                        results_count += results.len();
                    }
                    results_count
                })
            })
            .collect();

        let total_results: usize = handles
            .into_iter()
            .map(|h| h.join().expect("thread panicked"))
            .sum();

        // Each search should return 5 results
        let expected = num_threads * searches_per_thread * 5;
        assert_eq!(total_results, expected, "Expected {} total results", expected);
    }

    #[test]
    #[ignore]
    fn high_concurrency_searches() {
        let (vlite, _, paths) = setup_vlite_with_pool_size(20);

        create_collection_with_index(&vlite, "high_concurrent", 8, &paths.idx_path);

        // Insert data
        for i in 0..50 {
            let vector: Vec<f32> = (0..8).map(|j| (i + j) as f32 / 50.0).collect();
            let point = InsertPoint::builder()
                .collection_name("high_concurrent")
                .id(i)
                .vector(vector)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let num_threads = 16;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let vlite_clone = Arc::clone(&vlite);
                thread::spawn(move || {
                    let query: Vec<f32> = (0..8).map(|j| (thread_id + j) as f32 / 50.0).collect();
                    let search = SearchPoint::builder()
                        .collection_name("high_concurrent")
                        .vector(query)
                        .top_k(10)
                        .build()
                        .unwrap();

                    vlite_clone.search(search).expect("search").len()
                })
            })
            .collect();

        for handle in handles {
            let result_count = handle.join().expect("thread panicked");
            assert_eq!(result_count, 10, "Each search should return 10 results");
        }
    }
}

// ============================================================================
// Mixed Read/Write Tests
// ============================================================================

mod mixed_workload {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    #[ignore]
    fn concurrent_read_write() {
        let (vlite, _, paths) = setup_vlite_with_pool_size(15);

        create_collection_with_index(&vlite, "mixed_workload", 4, &paths.idx_path);

        // Insert initial data
        for i in 0..20 {
            let point = InsertPoint::builder()
                .collection_name("mixed_workload")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("initial insert");
        }

        let insert_counter = Arc::new(AtomicU64::new(100)); // Start new IDs at 100
        let num_writers = 2;
        let num_readers = 4;
        let operations_per_thread = 10;

        let mut handles = Vec::new();

        // Writer threads
        for _ in 0..num_writers {
            let vlite_clone = Arc::clone(&vlite);
            let counter = Arc::clone(&insert_counter);
            handles.push(thread::spawn(move || {
                for _ in 0..operations_per_thread {
                    let id = counter.fetch_add(1, Ordering::SeqCst);
                    let point = InsertPoint::builder()
                        .collection_name("mixed_workload")
                        .id(id)
                        .vector(vec![id as f32, 0.0, 0.0, 0.0])
                        .build()
                        .unwrap();
                    vlite_clone.insert(point).expect("concurrent insert");
                }
            }));
        }

        // Reader threads
        for thread_id in 0..num_readers {
            let vlite_clone = Arc::clone(&vlite);
            handles.push(thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let query_val = (thread_id * 10 + i) as f32;
                    let search = SearchPoint::builder()
                        .collection_name("mixed_workload")
                        .vector(vec![query_val, 0.0, 0.0, 0.0])
                        .top_k(5)
                        .build()
                        .unwrap();
                    vlite_clone.search(search).expect("concurrent search");
                }
            }));
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // Verify final state
        let final_count = insert_counter.load(Ordering::SeqCst) - 100;
        let expected_total = 20 + final_count as usize;

        let search = SearchPoint::builder()
            .collection_name("mixed_workload")
            .vector(vec![50.0, 0.0, 0.0, 0.0])
            .top_k(expected_total as i64)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("final search");
        assert!(
            results.len() >= 20,
            "Should have at least initial 20 records"
        );
    }

    #[test]
    #[ignore]
    fn stress_test_interleaved_operations() {
        let (vlite, _, paths) = setup_vlite_with_pool_size(10);

        create_collection_with_index(&vlite, "stress_test", 4, &paths.idx_path);

        let id_counter = Arc::new(AtomicU64::new(1));
        let num_threads = 6;
        let ops_per_thread = 15;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let vlite_clone = Arc::clone(&vlite);
                let counter = Arc::clone(&id_counter);
                thread::spawn(move || {
                    for i in 0..ops_per_thread {
                        // Alternate between insert and search
                        if i % 2 == 0 {
                            let id = counter.fetch_add(1, Ordering::SeqCst);
                            let point = InsertPoint::builder()
                                .collection_name("stress_test")
                                .id(id)
                                .vector(vec![id as f32, i as f32, 0.0, 0.0])
                                .build()
                                .unwrap();
                            let _ = vlite_clone.insert(point);
                        } else {
                            let search = SearchPoint::builder()
                                .collection_name("stress_test")
                                .vector(vec![i as f32, i as f32, 0.0, 0.0])
                                .top_k(5)
                                .build()
                                .unwrap();
                            let _ = vlite_clone.search(search);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // Just verify we can still use the database after stress
        let search = SearchPoint::builder()
            .collection_name("stress_test")
            .vector(vec![1.0, 1.0, 0.0, 0.0])
            .top_k(100)
            .build()
            .unwrap();

        let result = vlite.search(search);
        assert!(result.is_ok(), "Should be able to search after stress test");
    }
}

// ============================================================================
// Connection Pool Tests
// ============================================================================

mod connection_pool {
    use super::*;

    #[test]
    #[ignore]
    fn pool_exhaustion_handling() {
        // Create a pool with very limited connections
        let (vlite, _, paths) = setup_vlite_with_pool_size(2);

        create_collection_with_index(&vlite, "pool_test", 4, &paths.idx_path);

        // Insert some data
        for i in 0..10 {
            let point = InsertPoint::builder()
                .collection_name("pool_test")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Spawn more threads than pool connections
        let num_threads = 5;
        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let vlite_clone = Arc::clone(&vlite);
                thread::spawn(move || {
                    for _ in 0..5 {
                        let search = SearchPoint::builder()
                            .collection_name("pool_test")
                            .vector(vec![thread_id as f32, 0.0, 0.0, 0.0])
                            .top_k(5)
                            .build()
                            .unwrap();
                        vlite_clone.search(search).expect("search with limited pool");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should complete despite pool limits");
        }
    }

    #[test]
    #[ignore]
    fn large_pool_many_operations() {
        let (vlite, _, paths) = setup_vlite_with_pool_size(20);

        create_collection_with_index(&vlite, "large_pool", 4, &paths.idx_path);

        let num_threads = 15;
        let ops_per_thread = 20;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let vlite_clone = Arc::clone(&vlite);
                thread::spawn(move || {
                    for i in 0..ops_per_thread {
                        let id = (thread_id * 1000 + i) as u64;
                        let point = InsertPoint::builder()
                            .collection_name("large_pool")
                            .id(id)
                            .vector(vec![id as f32, 0.0, 0.0, 0.0])
                            .build()
                            .unwrap();
                        vlite_clone.insert(point).expect("insert with large pool");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        let search = SearchPoint::builder()
            .collection_name("large_pool")
            .vector(vec![0.0, 0.0, 0.0, 0.0])
            .top_k((num_threads * ops_per_thread) as i64)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), num_threads * ops_per_thread);
    }
}
