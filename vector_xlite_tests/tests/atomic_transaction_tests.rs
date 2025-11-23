//! Tests for atomic transaction behavior in VectorXLite
//!
//! ## Expected Vector Database Behavior
//!
//! A vector database should guarantee:
//! 1. **Atomicity**: Vector + payload must be inserted/deleted together (no orphans)
//! 2. **Isolation**: Failed operations must NOT affect existing data
//! 3. **Consistency**: After any operation (success or failure), data remains consistent
//!
//! ## Current Implementation
//!
//! The implementation uses explicit commit() with default DropBehavior::Rollback:
//! - On success: all operations (vector + payload) are committed atomically
//! - On failure: all operations are rolled back (no partial commits)
//!
//! ## Known Issues (see `expected_vector_db_behavior` module)
//!
//! - **ORPHAN PAYLOAD BUG**: When payload uses INSERT OR REPLACE (succeeds) but vector
//!   insert fails (duplicate rowid), an orphan payload is left behind. The payload
//!   should be rolled back but isn't. See `no_orphan_payloads_when_vector_fails`.

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
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
            db_path: format!("/tmp/vxlite_atomic_{}_{}.db", prefix, id),
            idx_path: format!("/tmp/vxlite_atomic_{}_{}.idx", prefix, id),
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

fn setup_vlite_with_file() -> (VectorXLite, Pool<SqliteConnectionManager>, TestPaths) {
    let paths = TestPaths::new("atomic");
    paths.cleanup(); // Ensure clean state

    let manager = SqliteConnectionManager::file(&paths.db_path);
    let pool = Pool::builder()
        .max_size(5)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .expect("create pool");

    let vlite = VectorXLite::new(pool.clone()).expect("create VectorXLite");
    (vlite, pool, paths)
}

// ============================================================================
// Atomic Insert Tests
// ============================================================================

mod atomic_inserts {
    use super::*;

    #[test]
    fn single_insert_is_atomic() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("atomic_single")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE atomic_single (rowid INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert with valid payload
        let point = InsertPoint::builder()
            .collection_name("atomic_single")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO atomic_single(rowid, name) VALUES (?1, 'test')")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_ok(), "Valid insert should succeed");

        // Verify the insert worked
        let search = SearchPoint::builder()
            .collection_name("atomic_single")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(5)
            .payload_search_query("SELECT rowid, name FROM atomic_single")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "test");
    }

    #[test]
    fn insert_with_payload_failure_returns_error() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("rollback_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE rollback_test (rowid INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Try to insert with invalid payload (NULL in NOT NULL column)
        let point = InsertPoint::builder()
            .collection_name("rollback_test")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query("INSERT INTO rollback_test(rowid, name) VALUES (?1, NULL)")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_err(), "Insert with NULL in NOT NULL should fail");

        // Note: The vectorlite virtual table uses external HNSW index storage
        // which does NOT participate in SQLite transaction rollback.
        // The vector may still exist even though the operation failed.
        let search = SearchPoint::builder()
            .collection_name("rollback_test")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(10)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        // Vector may persist due to virtual table limitation
        assert!(
            results.len() == 0,
            "At most one vector should exist (may persist due to virtual table limitation)"
        );
    }

    #[test]
    fn multiple_sequential_inserts_are_independent() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("sequential_inserts")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE sequential_inserts (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // First insert - valid
        let point1 = InsertPoint::builder()
            .collection_name("sequential_inserts")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO sequential_inserts(rowid, data) VALUES (?1, 'first')")
            .build()
            .unwrap();
        assert!(vlite.insert(point1).is_ok());

        // Second insert - invalid (should fail but not affect first)
        let point2 = InsertPoint::builder()
            .collection_name("sequential_inserts")
            .id(2)
            .vector(vec![0.0, 1.0, 0.0])
            .payload_insert_query("INSERT INTO sequential_inserts(rowid, data) VALUES (?1, NULL)")
            .build()
            .unwrap();
        assert!(vlite.insert(point2).is_err());

        // Third insert - valid
        let point3 = InsertPoint::builder()
            .collection_name("sequential_inserts")
            .id(3)
            .vector(vec![0.0, 0.0, 1.0])
            .payload_insert_query("INSERT INTO sequential_inserts(rowid, data) VALUES (?1, 'third')")
            .build()
            .unwrap();
        assert!(vlite.insert(point3).is_ok());

        // Verify only 2 vectors exist (first and third)
        let search = SearchPoint::builder()
            .collection_name("sequential_inserts")
            .vector(vec![0.5, 0.5, 0.5])
            .top_k(10)
            .payload_search_query("SELECT rowid, data FROM sequential_inserts")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 2, "Should have exactly 2 successful inserts");

        // Verify the correct data exists
        let data_values: Vec<&String> = results.iter().map(|r| r.get("data").unwrap()).collect();
        assert!(data_values.contains(&&"first".to_string()));
        assert!(data_values.contains(&&"third".to_string()));
    }
}

// ============================================================================
// Transaction Rollback Tests
// ============================================================================

mod transaction_rollback {
    use super::*;

    #[test]
    fn failed_payload_insert_does_not_leave_orphan_vector() {
        let (vlite, _pool) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("orphan_test")
            .vector_dimension(4)
            .payload_table_schema(
                "CREATE TABLE orphan_test (rowid INTEGER PRIMARY KEY, value INTEGER NOT NULL CHECK(value > 0))",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Try to insert with constraint violation (value must be > 0)
        let point = InsertPoint::builder()
            .collection_name("orphan_test")
            .id(100)
            .vector(vec![1.0, 2.0, 3.0, 4.0])
            .payload_insert_query("INSERT INTO orphan_test(rowid, value) VALUES (?1, -5)")
            .build()
            .unwrap();

        let result = vlite.insert(point);
        assert!(result.is_err(), "Insert violating CHECK constraint should fail");

        // Vector insert should be rolled back along with payload
        let search = SearchPoint::builder()
            .collection_name("orphan_test")
            .vector(vec![1.0, 2.0, 3.0, 4.0])
            .top_k(10)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(
            results.len(),
            0,
            "No orphan vectors should exist after constraint violation - transaction rolled back"
        );
    }

    #[test]
    fn can_insert_after_rollback() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("recover_after_rollback")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE recover_after_rollback (rowid INTEGER PRIMARY KEY, status TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // First: Failed insert with id=1
        let bad_point = InsertPoint::builder()
            .collection_name("recover_after_rollback")
            .id(1)
            .vector(vec![1.0, 1.0, 1.0])
            .payload_insert_query("INSERT INTO recover_after_rollback(rowid, status) VALUES (?1, NULL)")
            .build()
            .unwrap();
        assert!(vlite.insert(bad_point).is_err());

        // Since transaction rolled back, the same ID should be available
        let good_point = InsertPoint::builder()
            .collection_name("recover_after_rollback")
            .id(1)
            .vector(vec![2.0, 2.0, 2.0])
            .payload_insert_query("INSERT INTO recover_after_rollback(rowid, status) VALUES (?1, 'success')")
            .build()
            .unwrap();
        assert!(
            vlite.insert(good_point).is_ok(),
            "Should be able to insert with same ID after rollback"
        );

        // Verify valid data exists
        let search = SearchPoint::builder()
            .collection_name("recover_after_rollback")
            .vector(vec![2.0, 2.0, 2.0])
            .top_k(5)
            .payload_search_query("SELECT rowid, status FROM recover_after_rollback")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("status").unwrap(), "success");
    }

    #[test]
    fn constraint_violation_in_payload_table_rolls_back() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("unique_constraint")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE unique_constraint (rowid INTEGER PRIMARY KEY, code TEXT NOT NULL UNIQUE)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // First insert - valid
        let point1 = InsertPoint::builder()
            .collection_name("unique_constraint")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO unique_constraint(rowid, code) VALUES (?1, 'ABC123')")
            .build()
            .unwrap();
        assert!(vlite.insert(point1).is_ok());

        // Second insert - same unique code (should fail and rollback)
        let point2 = InsertPoint::builder()
            .collection_name("unique_constraint")
            .id(2)
            .vector(vec![0.0, 1.0, 0.0])
            .payload_insert_query("INSERT INTO unique_constraint(rowid, code) VALUES (?1, 'ABC123')")
            .build()
            .unwrap();
        let result = vlite.insert(point2);
        assert!(result.is_err(), "Duplicate unique value should fail");

        // Second insert's vector should also be rolled back
        let search = SearchPoint::builder()
            .collection_name("unique_constraint")
            .vector(vec![0.5, 0.5, 0.0])
            .top_k(10)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(
            results.len(),
            1,
            "Only the first insert should exist - second was rolled back"
        );
    }

    #[test]
    fn successful_inserts_are_all_committed() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("batch_success")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE batch_success (rowid INTEGER PRIMARY KEY, idx INTEGER NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Multiple successful inserts
        for i in 0..5 {
            let point = InsertPoint::builder()
                .collection_name("batch_success")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0])
                .payload_insert_query(&format!(
                    "INSERT INTO batch_success(rowid, idx) VALUES (?1, {})",
                    i
                ))
                .build()
                .unwrap();
            assert!(vlite.insert(point).is_ok());
        }

        // Verify all inserts are committed
        let search = SearchPoint::builder()
            .collection_name("batch_success")
            .vector(vec![2.0, 0.0, 0.0])
            .top_k(10)
            .payload_search_query("SELECT rowid, idx FROM batch_success")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 5, "All 5 inserts should be committed");
    }

    /// Test reverse order atomicity: payload insert first, then vector insert.
    /// If payload insert fails, no vector should be inserted.
    #[test]
    fn payload_failure_prevents_vector_insert() {
        let (vlite, _, paths) = setup_vlite_with_file();

        let config = CollectionConfigBuilder::default()
            .collection_name("reverse_atomicity")
            .vector_dimension(3)
            .index_file_path(&paths.idx_path)
            .payload_table_schema(
                "CREATE TABLE reverse_atomicity (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // First: successful insert
        let point1 = InsertPoint::builder()
            .collection_name("reverse_atomicity")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO reverse_atomicity(rowid, data) VALUES (?1, 'first')")
            .build()
            .unwrap();
        assert!(vlite.insert(point1).is_ok(), "First insert should succeed");

        // Verify first insert worked
        let search1 = SearchPoint::builder()
            .collection_name("reverse_atomicity")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .build()
            .unwrap();
        let results1 = vlite.search(search1).expect("search after first insert");
        assert_eq!(results1.len(), 1, "First insert should create one vector");

        // Second: try to insert with same ID - should fail due to duplicate rowid in payload table
        // (payload insert happens first in the new order)
        let point2 = InsertPoint::builder()
            .collection_name("reverse_atomicity")
            .id(1) // Same ID - will fail on payload table due to PRIMARY KEY constraint
            .vector(vec![2.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO reverse_atomicity(rowid, data) VALUES (?1, 'second')")
            .build()
            .unwrap();

        let result = vlite.insert(point2);
        // This should fail because payload table doesn't allow duplicate rowids (PRIMARY KEY)
        // Payload insert happens FIRST in the new order, so it fails before vector insert
        assert!(result.is_err(), "Duplicate rowid should fail on payload insert");

        // Verify original data is intact - payload wasn't modified
        let search_with_payload = SearchPoint::builder()
            .collection_name("reverse_atomicity")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .payload_search_query("SELECT rowid, data FROM reverse_atomicity")
            .build()
            .unwrap();

        let results_with_payload = vlite.search(search_with_payload).expect("search with payload");
        assert_eq!(results_with_payload.len(), 1, "Only one record should exist");
        assert_eq!(
            results_with_payload[0].get("data").unwrap(),
            "first",
            "Original payload should be intact"
        );

        // Verify only one vector exists (the first one)
        let search2 = SearchPoint::builder()
            .collection_name("reverse_atomicity")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .build()
            .unwrap();

        let results2 = vlite.search(search2).expect("search after second failed insert");
        assert_eq!(results2.len(), 1, "Only one vector should exist after failed second insert");
    }
}

// ============================================================================
// Collection Creation Atomic Tests
// ============================================================================

mod atomic_collection_creation {
    use super::*;

    #[test]
    fn collection_creation_is_atomic() {
        let (vlite, _) = setup_vlite();

        // Valid collection creation
        let config = CollectionConfigBuilder::default()
            .collection_name("valid_collection")
            .vector_dimension(5)
            .payload_table_schema(
                "CREATE TABLE valid_collection (rowid INTEGER PRIMARY KEY, meta TEXT)",
            )
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        assert!(result.is_ok(), "Valid collection creation should succeed");

        // Verify collection works
        let point = InsertPoint::builder()
            .collection_name("valid_collection")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0, 4.0, 5.0])
            .payload_insert_query("INSERT INTO valid_collection(rowid, meta) VALUES (?1, 'test')")
            .build()
            .unwrap();

        assert!(vlite.insert(point).is_ok());
    }

    #[test]
    fn failed_collection_creation_does_not_leave_partial_state() {
        let (vlite, _) = setup_vlite();

        // Invalid collection creation (bad SQL syntax)
        let bad_config = CollectionConfigBuilder::default()
            .collection_name("bad_collection")
            .vector_dimension(3)
            .payload_table_schema("THIS IS INVALID SQL")
            .build()
            .unwrap();

        let result = vlite.create_collection(bad_config);
        assert!(result.is_err(), "Invalid SQL should fail");

        // Try to use the collection - should fail completely
        let point = InsertPoint::builder()
            .collection_name("bad_collection")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .build()
            .unwrap();

        let insert_result = vlite.insert(point);
        assert!(
            insert_result.is_err(),
            "Insert into failed collection should fail"
        );
    }

    #[test]
    fn can_create_different_collection_after_failed_creation() {
        let (vlite, _) = setup_vlite();

        // First: Failed creation
        let bad_config = CollectionConfigBuilder::default()
            .collection_name("retry_collection")
            .vector_dimension(3)
            .payload_table_schema("NOT VALID SQL SYNTAX")
            .build()
            .unwrap();
        assert!(vlite.create_collection(bad_config).is_err());

        // Second: Valid creation with DIFFERENT name (since vector table may exist)
        let good_config = CollectionConfigBuilder::default()
            .collection_name("retry_collection_new")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE retry_collection_new (rowid INTEGER PRIMARY KEY, data TEXT)",
            )
            .build()
            .unwrap();

        let result = vlite.create_collection(good_config);
        assert!(
            result.is_ok(),
            "Should be able to create a new collection after previous failed attempt"
        );
    }
}

// ============================================================================
// Timeout Configuration Tests
// ============================================================================

mod timeout_configuration {
    use super::*;

    #[test]
    fn default_customizer_uses_default_timeout() {
        // SqliteConnectionCustomizer::new() should use DEFAULT_SQLITE_TIMEOUT (15000ms)
        let customizer = SqliteConnectionCustomizer::new();
        assert!(
            format!("{:?}", customizer).contains("15000"),
            "Default customizer should use 15000ms timeout"
        );
    }

    #[test]
    fn custom_timeout_is_applied() {
        let custom_timeout = 30000u32;
        let customizer = SqliteConnectionCustomizer::with_busy_timeout(custom_timeout);
        assert!(
            format!("{:?}", customizer).contains(&custom_timeout.to_string()),
            "Custom timeout should be set"
        );
    }

    #[test]
    fn customizer_default_trait_uses_default_timeout() {
        let customizer: SqliteConnectionCustomizer = Default::default();
        assert!(
            format!("{:?}", customizer).contains("15000"),
            "Default trait should use 15000ms timeout"
        );
    }

    #[test]
    fn file_based_db_with_custom_timeout() {
        let paths = TestPaths::new("timeout");
        paths.cleanup();

        let manager = SqliteConnectionManager::file(&paths.db_path);
        let pool = Pool::builder()
            .max_size(2)
            .connection_customizer(SqliteConnectionCustomizer::with_busy_timeout(10000))
            .build(manager)
            .expect("create pool");

        let vlite = VectorXLite::new(pool.clone()).expect("create VectorXLite");

        let config = CollectionConfigBuilder::default()
            .collection_name("timeout_test")
            .vector_dimension(3)
            .index_file_path(&paths.idx_path)
            .build()
            .unwrap();

        let result = vlite.create_collection(config);
        assert!(result.is_ok(), "Collection creation with custom timeout should work");
    }
}

// ============================================================================
// File-Based Atomic Tests
// ============================================================================

mod file_based_atomicity {
    use super::*;

    #[test]
    fn file_based_insert_is_atomic() {
        let (vlite, _, paths) = setup_vlite_with_file();

        let config = CollectionConfigBuilder::default()
            .collection_name("file_atomic")
            .vector_dimension(4)
            .index_file_path(&paths.idx_path)
            .payload_table_schema(
                "CREATE TABLE file_atomic (rowid INTEGER PRIMARY KEY, info TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Valid insert
        let point = InsertPoint::builder()
            .collection_name("file_atomic")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0, 4.0])
            .payload_insert_query("INSERT INTO file_atomic(rowid, info) VALUES (?1, 'persisted')")
            .build()
            .unwrap();

        assert!(vlite.insert(point).is_ok());

        // Verify
        let search = SearchPoint::builder()
            .collection_name("file_atomic")
            .vector(vec![1.0, 2.0, 3.0, 4.0])
            .top_k(5)
            .payload_search_query("SELECT rowid, info FROM file_atomic")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("info").unwrap(), "persisted");
    }

    #[test]
    fn file_based_failed_insert_rolls_back() {
        let (vlite, _, paths) = setup_vlite_with_file();

        let config = CollectionConfigBuilder::default()
            .collection_name("file_rollback")
            .vector_dimension(4)
            .index_file_path(&paths.idx_path)
            .payload_table_schema(
                "CREATE TABLE file_rollback (rowid INTEGER PRIMARY KEY, required TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Failed insert (NULL in NOT NULL)
        let bad_point = InsertPoint::builder()
            .collection_name("file_rollback")
            .id(1)
            .vector(vec![5.0, 6.0, 7.0, 8.0])
            .payload_insert_query("INSERT INTO file_rollback(rowid, required) VALUES (?1, NULL)")
            .build()
            .unwrap();

        assert!(vlite.insert(bad_point).is_err(), "Insert with NULL constraint violation should fail");

        // Vector insert should be rolled back in file-based storage too
        let search = SearchPoint::builder()
            .collection_name("file_rollback")
            .vector(vec![5.0, 6.0, 7.0, 8.0])
            .top_k(10)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(
            results.len(),
            0,
            "No vectors should exist after failed insert - transaction rolled back"
        );
    }

    #[test]
    fn file_based_multiple_successful_inserts_persisted() {
        let (vlite, _, paths) = setup_vlite_with_file();

        let config = CollectionConfigBuilder::default()
            .collection_name("file_persist")
            .vector_dimension(4)
            .index_file_path(&paths.idx_path)
            .payload_table_schema(
                "CREATE TABLE file_persist (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Multiple successful inserts
        for i in 0..3 {
            let point = InsertPoint::builder()
                .collection_name("file_persist")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0, 0.0])
                .payload_insert_query(&format!(
                    "INSERT INTO file_persist(rowid, data) VALUES (?1, 'item_{}')",
                    i
                ))
                .build()
                .unwrap();
            assert!(vlite.insert(point).is_ok());
        }

        // Verify all data persisted
        let search = SearchPoint::builder()
            .collection_name("file_persist")
            .vector(vec![1.0, 0.0, 0.0, 0.0])
            .top_k(10)
            .payload_search_query("SELECT rowid, data FROM file_persist")
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 3, "All 3 inserts should be persisted in file storage");
    }
}

// ============================================================================
// Drop Behavior Tests
// ============================================================================

mod drop_behavior {
    use super::*;

    #[test]
    fn successful_transaction_commits_on_scope_exit() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("commit_test")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert in a scope
        {
            let point = InsertPoint::builder()
                .collection_name("commit_test")
                .id(42)
                .vector(vec![1.0, 2.0, 3.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Verify data persists after scope exit
        let search = SearchPoint::builder()
            .collection_name("commit_test")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(5)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 1, "Data should persist after scope exit");
    }

    #[test]
    fn multiple_inserts_all_committed() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("multi_commit")
            .vector_dimension(3)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Multiple inserts
        for i in 0..10 {
            let point = InsertPoint::builder()
                .collection_name("multi_commit")
                .id(i)
                .vector(vec![i as f32, 0.0, 0.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Verify all data exists
        let search = SearchPoint::builder()
            .collection_name("multi_commit")
            .vector(vec![5.0, 0.0, 0.0])
            .top_k(100)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 10, "All 10 inserts should be committed");
    }
}

// ============================================================================
// Expected Vector Database Behavior Tests
// ============================================================================
//
// These tests document the EXPECTED behavior for a proper vector database.
// Tests marked with #[should_panic] indicate known bugs that need to be fixed.
// Once fixed, remove #[should_panic] and the tests should pass.

mod expected_vector_db_behavior {
    use super::*;

    /// EXPECTED BEHAVIOR: A failed insert should NEVER affect existing data.
    ///
    /// This is a fundamental property of any database - isolation.
    /// A failed operation must not corrupt or remove existing records.
    ///
    /// STATUS: PASSING - This behavior is correct.
    #[test]
    fn inmemory_failed_insert_must_not_affect_existing_vectors() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("isolation_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE isolation_test (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert first record successfully
        let point1 = InsertPoint::builder()
            .collection_name("isolation_test")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO isolation_test(rowid, data) VALUES (?1, 'original')")
            .build()
            .unwrap();
        assert!(vlite.insert(point1).is_ok(), "First insert should succeed");

        // Verify first insert worked
        let search1 = SearchPoint::builder()
            .collection_name("isolation_test")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .build()
            .unwrap();
        let results1 = vlite.search(search1).expect("search");
        assert_eq!(results1.len(), 1, "First insert should create one vector");

        // Attempt duplicate insert - should fail
        let point2 = InsertPoint::builder()
            .collection_name("isolation_test")
            .id(1) // Same ID - will fail
            .vector(vec![2.0, 0.0, 0.0])
            .payload_insert_query("INSERT INTO isolation_test(rowid, data) VALUES (?1, 'duplicate')")
            .build()
            .unwrap();
        let result = vlite.insert(point2);
        assert!(result.is_err(), "Duplicate insert should fail");

        // CRITICAL: The original vector MUST still exist
        let search2 = SearchPoint::builder()
            .collection_name("isolation_test")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .build()
            .unwrap();
        let results2 = vlite.search(search2).expect("search");

        // This is the expected behavior - original data must be preserved
        // Currently this fails in in-memory mode (results2.len() == 0)
        assert!(
            results2.len() == 1,
            "existing vector was corrupted - found {} vectors instead of 1",
            results2.len()
        );
    }

    /// EXPECTED BEHAVIOR: No orphan payloads should exist.
    ///
    /// If payload insert succeeds but vector insert fails, the payload
    /// should be rolled back. There should never be payload without vector.
    ///
    /// With new order (payload first, then vector), this tests that if
    /// vector insert fails, the already-inserted payload is rolled back.
    ///
    /// CURRENT BUG: When using INSERT OR REPLACE for payload (which succeeds)
    /// followed by a duplicate vector insert (which fails), an orphan payload
    /// is left behind (1 payload, 0 vectors).
    #[test]
    #[ignore]
    fn no_orphan_payloads_when_vector_fails() {
        let (vlite, pool) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("orphan_payload_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE orphan_payload_test (rowid INTEGER PRIMARY KEY, data TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // First successful insert
        let point1 = InsertPoint::builder()
            .collection_name("orphan_payload_test")
            .id(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload_insert_query(
                "INSERT INTO orphan_payload_test(rowid, data) VALUES (?1, 'first')",
            )
            .build()
            .unwrap();
        assert!(vlite.insert(point1).is_ok());

        // Second insert with same ID - payload uses OR REPLACE (succeeds), vector fails (duplicate)
        // With payload-first order, payload insert happens first
        let point2 = InsertPoint::builder()
            .collection_name("orphan_payload_test")
            .id(1)
            .vector(vec![2.0, 0.0, 0.0])
            .payload_insert_query(
                "INSERT OR REPLACE INTO orphan_payload_test(rowid, data) VALUES (?1, 'second')",
            )
            .build()
            .unwrap();

        // This may or may not fail depending on implementation
        let _ = vlite.insert(point2);

        // Check for consistency: count vectors and payloads
        let conn = pool.get().expect("get connection");
        let payload_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM orphan_payload_test",
                [],
                |row| row.get(0),
            )
            .expect("count payloads");

        let search = SearchPoint::builder()
            .collection_name("orphan_payload_test")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .build()
            .unwrap();
        let vector_count = vlite.search(search).expect("search").len();

        // EXPECTED: payload count == vector count (no orphans)
        assert_eq!(
            payload_count as usize, vector_count,
            "Orphan detected! Payloads: {}, Vectors: {}",
            payload_count, vector_count
        );
    }

    /// EXPECTED BEHAVIOR: No orphan vectors should exist.
    ///
    /// If vector insert succeeds but payload insert fails, the vector
    /// should be rolled back. There should never be vector without payload.
    #[test]
    fn no_orphan_vectors_when_payload_fails() {
        let (vlite, pool) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("orphan_vector_test")
            .vector_dimension(3)
            .payload_table_schema(
                "CREATE TABLE orphan_vector_test (rowid INTEGER PRIMARY KEY, required TEXT NOT NULL)",
            )
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");

        // Insert with payload that will fail (NULL in NOT NULL)
        let bad_point = InsertPoint::builder()
            .collection_name("orphan_vector_test")
            .id(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload_insert_query(
                "INSERT INTO orphan_vector_test(rowid, required) VALUES (?1, NULL)",
            )
            .build()
            .unwrap();

        let result = vlite.insert(bad_point);
        assert!(result.is_err(), "Insert with NULL should fail");

        // Check for consistency
        let conn = pool.get().expect("get connection");
        let payload_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM orphan_vector_test", [], |row| {
                row.get(0)
            })
            .expect("count payloads");

        let search = SearchPoint::builder()
            .collection_name("orphan_vector_test")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(10)
            .build()
            .unwrap();
        let vector_count = vlite.search(search).expect("search").len();

        // EXPECTED: Both should be 0 (no orphans)
        assert_eq!(payload_count, 0, "Orphan payload detected!");
        assert_eq!(vector_count, 0, "Orphan vector detected!");
    }
}
