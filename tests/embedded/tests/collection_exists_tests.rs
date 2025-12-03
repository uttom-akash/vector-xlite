//! Tests for collection_exists method in VectorXLite
//!
//! These tests verify:
//! - Returns true when a collection exists
//! - Returns false when a collection does not exist
//! - Handles edge cases properly

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
// Basic Functionality Tests
// ============================================================================

#[test]
fn collection_exists_returns_false_for_nonexistent_collection() {
    let (vlite, _) = setup_vlite();

    let exists = vlite
        .collection_exists("nonexistent_collection")
        .expect("check should succeed");

    assert!(!exists, "Nonexistent collection should return false");
}

#[test]
fn collection_exists_returns_true_after_creation() {
    let (vlite, _) = setup_vlite();

    // Create a collection
    let config = CollectionConfig::builder()
        .collection_name("test_collection")
        .vector_dimension(3)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection creation should succeed");

    // Check if it exists
    let exists = vlite
        .collection_exists("test_collection")
        .expect("check should succeed");

    assert!(exists, "Created collection should exist");
}

#[test]
fn collection_exists_returns_false_before_creation() {
    let (vlite, _) = setup_vlite();

    // Check before creation
    let exists_before = vlite
        .collection_exists("future_collection")
        .expect("check should succeed");

    assert!(
        !exists_before,
        "Collection should not exist before creation"
    );

    // Create the collection
    let config = CollectionConfig::builder()
        .collection_name("future_collection")
        .vector_dimension(5)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection creation should succeed");

    // Check after creation
    let exists_after = vlite
        .collection_exists("future_collection")
        .expect("check should succeed");

    assert!(exists_after, "Collection should exist after creation");
}

// ============================================================================
// Multiple Collections Tests
// ============================================================================

#[test]
fn collection_exists_handles_multiple_collections() {
    let (vlite, _) = setup_vlite();

    // Create first collection
    let config1 = CollectionConfig::builder()
        .collection_name("collection_1")
        .vector_dimension(3)
        .build()
        .unwrap();

    vlite
        .create_collection(config1)
        .expect("first collection creation should succeed");

    // Create second collection
    let config2 = CollectionConfig::builder()
        .collection_name("collection_2")
        .vector_dimension(5)
        .build()
        .unwrap();

    vlite
        .create_collection(config2)
        .expect("second collection creation should succeed");

    // Check both exist
    assert!(
        vlite
            .collection_exists("collection_1")
            .expect("check should succeed"),
        "First collection should exist"
    );
    assert!(
        vlite
            .collection_exists("collection_2")
            .expect("check should succeed"),
        "Second collection should exist"
    );

    // Check non-existent collection
    assert!(
        !vlite
            .collection_exists("collection_3")
            .expect("check should succeed"),
        "Third collection should not exist"
    );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn collection_exists_with_special_characters() {
    let (vlite, _) = setup_vlite();

    // Test with underscores
    let config = CollectionConfig::builder()
        .collection_name("test_collection_with_underscores")
        .vector_dimension(3)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection creation should succeed");

    assert!(
        vlite
            .collection_exists("test_collection_with_underscores")
            .expect("check should succeed"),
        "Collection with underscores should exist"
    );
}

#[test]
fn collection_exists_case_sensitive() {
    let (vlite, _) = setup_vlite();

    // Create a collection with lowercase name
    let config = CollectionConfig::builder()
        .collection_name("lowercase_collection")
        .vector_dimension(3)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection creation should succeed");

    // Check with exact case
    assert!(
        vlite
            .collection_exists("lowercase_collection")
            .expect("check should succeed"),
        "Collection with exact case should exist"
    );

    // SQLite table names are case-sensitive when quoted
    // This test documents the current behavior
    let exists_uppercase = vlite
        .collection_exists("LOWERCASE_COLLECTION")
        .expect("check should succeed");

    // Note: SQLite treats table names as case-sensitive for unquoted identifiers
    assert!(
        !exists_uppercase,
        "Collection names are case-sensitive"
    );
}

#[test]
fn collection_exists_empty_string() {
    let (vlite, _) = setup_vlite();

    // Check with empty string
    let exists = vlite
        .collection_exists("")
        .expect("check should succeed");

    assert!(!exists, "Empty string should not match any collection");
}

// ============================================================================
// Use Case: Preventing Duplicate Creation
// ============================================================================

#[test]
fn use_case_prevent_duplicate_collection_creation() {
    let (vlite, _) = setup_vlite();

    let collection_name = "products";

    // First time: collection doesn't exist, so create it
    if !vlite
        .collection_exists(collection_name)
        .expect("check should succeed")
    {
        let config = CollectionConfig::builder()
            .collection_name(collection_name)
            .vector_dimension(128)
            .build()
            .unwrap();

        vlite
            .create_collection(config)
            .expect("first creation should succeed");
    }

    // Second time: collection exists, so skip creation
    let should_create = !vlite
        .collection_exists(collection_name)
        .expect("check should succeed");

    assert!(
        !should_create,
        "Should not attempt to create duplicate collection"
    );

    // Attempting to create again would fail, but we prevented it
    // This demonstrates the practical use case
}

// ============================================================================
// Integration with Other Operations
// ============================================================================

#[test]
fn collection_exists_after_insert_operations() {
    let (vlite, _) = setup_vlite();

    let collection_name = "insert_test";

    // Create collection
    let config = CollectionConfig::builder()
        .collection_name(collection_name)
        .vector_dimension(3)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection creation should succeed");

    // Insert some data with explicit ID
    let point = InsertPoint::builder()
        .collection_name(collection_name)
        .vector(vec![1.0, 2.0, 3.0])
        .id(1)
        .build()
        .unwrap();

    vlite.insert(point).expect("insert should succeed");

    // Collection should still exist after insert
    assert!(
        vlite
            .collection_exists(collection_name)
            .expect("check should succeed"),
        "Collection should exist after insert operations"
    );
}
