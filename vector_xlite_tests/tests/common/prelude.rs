//! # Test Prelude
//!
//! Convenient re-exports for test modules. Import everything you need with:
//!
//! ```rust
//! use common::prelude::*;
//! ```

// Re-export core types
pub use super::{
    Collection, CollectionBuilder, InsertBuilder, SearchBuilder, StorageMode, TestContext,
    TestPaths, unique_id, unique_name,
    // Legacy compatibility
    setup_vlite, setup_vlite_with_file,
};

// Re-export fixtures
pub use super::fixtures;

// Re-export generators
pub use super::generators::{
    // Vector generators
    random_vector, seeded_vector, zero_vector, ones_vector, unit_vector,
    axis_vector, constant_vector, sequential_vector, normalized_sequential,
    // Batch generators
    vector_batch, seeded_batch, vectors_with_ids, vectors_with_ids_from,
    // Special patterns
    clustered_vectors, linear_vectors, circular_vectors, orthogonal_vectors,
    // Vector operations
    normalize, l2_distance, cosine_similarity, inner_product, add_noise, perturb,
    // ID generators
    sequential_ids, ids_from, scattered_ids,
};

// Re-export assertion helpers
pub use super::assertions::{
    SearchResults, contains_rowid, get_rowids, first_field,
    is_sorted_asc, is_sorted_desc, get_field_values, count_matching,
    assert_vectors_approx_eq, assert_vector_dimension, assert_vector_normalized,
};

// Re-export VectorXLite types
pub use vector_xlite::{
    customizer::SqliteConnectionCustomizer,
    types::*,
    VectorXLite,
};

// Re-export r2d2 types
pub use r2d2::Pool;
pub use r2d2_sqlite::SqliteConnectionManager;

// Re-export assertion macros
pub use crate::{
    assert_result_count, assert_not_empty, assert_empty, assert_at_least, assert_at_most,
    assert_field, assert_first_result, assert_contains_rowid, assert_not_contains_rowid,
    assert_ok, assert_err, assert_err_contains,
    assert_collection_count, assert_collection_empty, assert_collection_not_empty,
};
