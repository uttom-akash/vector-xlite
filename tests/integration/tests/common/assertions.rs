//! # Test Assertion Helpers
//!
//! Custom assertion macros and helpers for VectorXLite tests.
//!
//! ## Usage
//!
//! ```rust
//! use common::assertions::*;
//!
//! // Check result count
//! assert_result_count!(results, 5);
//!
//! // Check result contains specific ID
//! assert_contains_id!(results, 42);
//!
//! // Check first result
//! assert_first_result!(results, "name", "Alice");
//! ```

use std::collections::HashMap;

/// Type alias for search results
pub type SearchResults = Vec<HashMap<String, String>>;

// ============================================================================
// Result Count Assertions
// ============================================================================

/// Assert that results have exactly the expected count
#[macro_export]
macro_rules! assert_result_count {
    ($results:expr, $expected:expr) => {
        assert_eq!(
            $results.len(),
            $expected,
            "Expected {} results, got {}",
            $expected,
            $results.len()
        );
    };
    ($results:expr, $expected:expr, $msg:expr) => {
        assert_eq!($results.len(), $expected, $msg);
    };
}

/// Assert that results are not empty
#[macro_export]
macro_rules! assert_not_empty {
    ($results:expr) => {
        assert!(
            !$results.is_empty(),
            "Expected non-empty results, got empty"
        );
    };
    ($results:expr, $msg:expr) => {
        assert!(!$results.is_empty(), $msg);
    };
}

/// Assert that results are empty
#[macro_export]
macro_rules! assert_empty {
    ($results:expr) => {
        assert!(
            $results.is_empty(),
            "Expected empty results, got {} results",
            $results.len()
        );
    };
    ($results:expr, $msg:expr) => {
        assert!($results.is_empty(), $msg);
    };
}

/// Assert that results have at least the expected count
#[macro_export]
macro_rules! assert_at_least {
    ($results:expr, $min:expr) => {
        assert!(
            $results.len() >= $min,
            "Expected at least {} results, got {}",
            $min,
            $results.len()
        );
    };
}

/// Assert that results have at most the expected count
#[macro_export]
macro_rules! assert_at_most {
    ($results:expr, $max:expr) => {
        assert!(
            $results.len() <= $max,
            "Expected at most {} results, got {}",
            $max,
            $results.len()
        );
    };
}

// ============================================================================
// Result Content Assertions
// ============================================================================

/// Assert that a result has a specific field value
#[macro_export]
macro_rules! assert_field {
    ($result:expr, $field:expr, $expected:expr) => {
        let value = $result.get($field).expect(&format!("Field '{}' not found", $field));
        assert_eq!(
            value, $expected,
            "Field '{}': expected '{}', got '{}'",
            $field, $expected, value
        );
    };
}

/// Assert that first result has a specific field value
#[macro_export]
macro_rules! assert_first_result {
    ($results:expr, $field:expr, $expected:expr) => {
        assert!(!$results.is_empty(), "Results are empty, cannot check first result");
        let value = $results[0].get($field).expect(&format!("Field '{}' not found", $field));
        assert_eq!(
            value, $expected,
            "First result field '{}': expected '{}', got '{}'",
            $field, $expected, value
        );
    };
}

/// Assert that results contain a row with specific rowid
#[macro_export]
macro_rules! assert_contains_rowid {
    ($results:expr, $rowid:expr) => {
        let found = $results.iter().any(|r| {
            r.get("rowid")
                .map(|v| v.parse::<u64>().ok() == Some($rowid))
                .unwrap_or(false)
        });
        assert!(found, "Results do not contain rowid {}", $rowid);
    };
}

/// Assert that results do NOT contain a row with specific rowid
#[macro_export]
macro_rules! assert_not_contains_rowid {
    ($results:expr, $rowid:expr) => {
        let found = $results.iter().any(|r| {
            r.get("rowid")
                .map(|v| v.parse::<u64>().ok() == Some($rowid))
                .unwrap_or(false)
        });
        assert!(!found, "Results should not contain rowid {}", $rowid);
    };
}

// ============================================================================
// Functional Assertions
// ============================================================================

/// Check if results contain a specific rowid
pub fn contains_rowid(results: &SearchResults, rowid: u64) -> bool {
    results.iter().any(|r| {
        r.get("rowid")
            .map(|v| v.parse::<u64>().ok() == Some(rowid))
            .unwrap_or(false)
    })
}

/// Get all rowids from results
pub fn get_rowids(results: &SearchResults) -> Vec<u64> {
    results
        .iter()
        .filter_map(|r| r.get("rowid")?.parse().ok())
        .collect()
}

/// Get a specific field from first result
pub fn first_field(results: &SearchResults, field: &str) -> Option<String> {
    results.first()?.get(field).cloned()
}

/// Check if results are sorted by a numeric field (ascending)
pub fn is_sorted_asc(results: &SearchResults, field: &str) -> bool {
    let values: Vec<f64> = results
        .iter()
        .filter_map(|r| r.get(field)?.parse().ok())
        .collect();

    values.windows(2).all(|w| w[0] <= w[1])
}

/// Check if results are sorted by a numeric field (descending)
pub fn is_sorted_desc(results: &SearchResults, field: &str) -> bool {
    let values: Vec<f64> = results
        .iter()
        .filter_map(|r| r.get(field)?.parse().ok())
        .collect();

    values.windows(2).all(|w| w[0] >= w[1])
}

/// Get all values for a field
pub fn get_field_values(results: &SearchResults, field: &str) -> Vec<String> {
    results
        .iter()
        .filter_map(|r| r.get(field).cloned())
        .collect()
}

/// Count results matching a predicate
pub fn count_matching<F>(results: &SearchResults, predicate: F) -> usize
where
    F: Fn(&HashMap<String, String>) -> bool,
{
    results.iter().filter(|r| predicate(r)).count()
}

// ============================================================================
// Error Assertions
// ============================================================================

/// Assert that an operation succeeds
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        assert!($result.is_ok(), "Expected Ok, got Err: {:?}", $result.err());
    };
    ($result:expr, $msg:expr) => {
        assert!($result.is_ok(), "{}: {:?}", $msg, $result.err());
    };
}

/// Assert that an operation fails
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        assert!($result.is_err(), "Expected Err, got Ok");
    };
    ($result:expr, $msg:expr) => {
        assert!($result.is_err(), $msg);
    };
}

/// Assert that an operation fails with a specific error message substring
#[macro_export]
macro_rules! assert_err_contains {
    ($result:expr, $substring:expr) => {
        assert!($result.is_err(), "Expected Err, got Ok");
        let err_msg = format!("{:?}", $result.err().unwrap());
        assert!(
            err_msg.contains($substring),
            "Error message '{}' does not contain '{}'",
            err_msg,
            $substring
        );
    };
}

// ============================================================================
// Collection State Assertions
// ============================================================================

/// Assert collection has expected vector count
#[macro_export]
macro_rules! assert_collection_count {
    ($collection:expr, $expected:expr) => {
        let count = $collection.count();
        assert_eq!(
            count, $expected,
            "Collection '{}' has {} vectors, expected {}",
            $collection.name, count, $expected
        );
    };
}

/// Assert collection is empty
#[macro_export]
macro_rules! assert_collection_empty {
    ($collection:expr) => {
        assert!(
            $collection.is_empty(),
            "Collection '{}' should be empty but has {} vectors",
            $collection.name,
            $collection.count()
        );
    };
}

/// Assert collection is not empty
#[macro_export]
macro_rules! assert_collection_not_empty {
    ($collection:expr) => {
        assert!(
            !$collection.is_empty(),
            "Collection '{}' should not be empty",
            $collection.name
        );
    };
}

// ============================================================================
// Vector Assertions
// ============================================================================

/// Assert two vectors are approximately equal
pub fn assert_vectors_approx_eq(a: &[f32], b: &[f32], epsilon: f32) {
    assert_eq!(a.len(), b.len(), "Vector lengths differ: {} vs {}", a.len(), b.len());
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            (x - y).abs() < epsilon,
            "Vectors differ at index {}: {} vs {} (epsilon: {})",
            i, x, y, epsilon
        );
    }
}

/// Assert vector has expected dimension
pub fn assert_vector_dimension(v: &[f32], expected_dim: usize) {
    assert_eq!(
        v.len(),
        expected_dim,
        "Vector dimension is {}, expected {}",
        v.len(),
        expected_dim
    );
}

/// Assert vector is normalized (unit length)
pub fn assert_vector_normalized(v: &[f32], epsilon: f32) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (norm - 1.0).abs() < epsilon,
        "Vector norm is {}, expected 1.0 (epsilon: {})",
        norm,
        epsilon
    );
}
