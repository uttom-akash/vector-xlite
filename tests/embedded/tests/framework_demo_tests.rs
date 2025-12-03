//! # Test Framework Demo
//!
//! This file demonstrates how to use the new testing framework.
//! Use these examples as templates for writing new tests.

mod common;

use common::prelude::*;

// ============================================================================
// Basic Tests - Quick Setup
// ============================================================================

mod basic_usage {
    use super::*;

    #[test]
    fn simple_insert_and_search() {
        // One-line setup
        let ctx = TestContext::memory();
        let coll = ctx.quick_collection("simple");

        // Insert a vector
        coll.insert_vector(1, vec![1.0, 0.0, 0.0]);

        // Search
        let results = coll.search_quick(vec![1.0, 0.0, 0.0]);

        assert_result_count!(results, 1);
    }

    #[test]
    fn using_collection_builder() {
        let ctx = TestContext::memory();

        // Fluent collection creation
        let coll = ctx
            .collection("products")
            .dimension(128)
            .cosine()
            .max_elements(10_000)
            .create();

        // Insert with generated vector
        let vector = random_vector(128);
        coll.insert_vector(1, vector.clone());

        // Search returns the vector
        let results = coll.search(vector).top_k(5).execute();
        assert_result_count!(results, 1);
    }

    #[test]
    fn with_payload_support() {
        let ctx = TestContext::memory();

        // Collection with payload columns
        let coll = ctx
            .collection("users")
            .dimension(3)
            .with_payload("name TEXT NOT NULL, role TEXT")
            .create();

        // Insert with payload using fluent API
        coll.insert(1)
            .vector(vec![1.0, 0.0, 0.0])
            .payload(&format!(
                "INSERT INTO {} (rowid, name, role) VALUES (?1, 'Alice', 'admin')",
                coll.name
            ))
            .execute_ok();

        // Search with payload query
        let results = coll
            .search(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .payload(&format!("SELECT rowid, name, role FROM {}", coll.name))
            .execute();

        assert_result_count!(results, 1);
        assert_first_result!(results, "name", "Alice");
        assert_first_result!(results, "role", "admin");
    }
}

// ============================================================================
// Using Fixtures
// ============================================================================

mod using_fixtures {
    use super::*;

    #[test]
    fn simple_3d_fixture() {
        let ctx = TestContext::memory();

        // Use predefined fixture
        let coll = fixtures::simple_3d(&ctx);

        coll.insert_vector(1, vec![1.0, 2.0, 3.0]);

        assert!(!coll.is_empty());
    }

    #[test]
    fn pre_populated_fixture() {
        let ctx = TestContext::memory();

        // Fixture already has 10 vectors
        let coll = fixtures::populated_10(&ctx);

        assert_collection_count!(coll, 10);
    }

    #[test]
    fn similarity_scenario() {
        let ctx = TestContext::memory();

        // Get a pre-built test scenario
        let scenario = fixtures::similarity_scenario(&ctx);

        let results = scenario
            .collection
            .search(scenario.query_vector)
            .top_k(1)
            .execute();

        assert_result_count!(results, 1);
        // First result should be the expected nearest neighbor
    }

    #[test]
    fn filter_scenario() {
        let ctx = TestContext::memory();

        let scenario = fixtures::filter_scenario(&ctx);

        let results = scenario
            .collection
            .search(vec![3.0, 0.0, 0.0])
            .top_k(100)
            .payload(&scenario.filter_sql)
            .execute();

        assert_result_count!(results, scenario.expected_count);
    }
}

// ============================================================================
// Using Generators
// ============================================================================

mod using_generators {
    use super::*;

    #[test]
    fn random_vectors() {
        let ctx = TestContext::memory();
        let coll = fixtures::high_dim_128(&ctx);

        // Generate and insert random vectors
        for id in 1..=10 {
            let vec = random_vector(128);
            coll.insert_vector(id, vec);
        }

        assert_collection_count!(coll, 10);
    }

    #[test]
    fn reproducible_vectors() {
        let ctx = TestContext::memory();
        let coll = ctx.quick_collection("seeded");

        // Seeded vectors are reproducible
        let v1 = seeded_vector(3, 42);
        let v2 = seeded_vector(3, 42);

        assert_eq!(v1, v2);

        coll.insert_vector(1, v1);
        let results = coll.search(v2).top_k(1).execute();
        assert_result_count!(results, 1);
    }

    #[test]
    fn batch_insert() {
        let ctx = TestContext::memory();
        let coll = ctx.quick_collection("batch");

        // Generate batch with IDs
        let vectors = vectors_with_ids(100, 3);
        coll.insert_vectors(vectors);

        assert_collection_count!(coll, 100);
    }

    #[test]
    fn clustered_data() {
        let ctx = TestContext::memory();
        let coll = ctx.quick_collection("clusters");

        // Generate 3 clusters of 10 points each
        let data = clustered_vectors(3, 10, 3);
        coll.insert_vectors(data);

        assert_collection_count!(coll, 30);
    }

    #[test]
    fn vector_operations() {
        // Test vector math utilities
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];

        // L2 distance
        let dist = l2_distance(&v1, &v2);
        assert!((dist - std::f32::consts::SQRT_2).abs() < 0.001);

        // Cosine similarity (orthogonal vectors = 0)
        let sim = cosine_similarity(&v1, &v2);
        assert!(sim.abs() < 0.001);

        // Normalize
        let v3 = vec![3.0, 4.0, 0.0];
        let normalized = normalize(&v3);
        assert_vector_normalized(&normalized, 0.001);
    }
}

// ============================================================================
// File-based Storage Tests
// ============================================================================

mod file_storage {
    use super::*;

    #[test]
    fn file_backed_context() {
        // Context with file storage (auto-cleanup on drop)
        let ctx = TestContext::file();

        let coll = ctx
            .collection("persistent")
            .dimension(3)
            .create();

        coll.insert_vector(1, vec![1.0, 2.0, 3.0]);

        let results = coll.search_quick(vec![1.0, 2.0, 3.0]);
        assert_result_count!(results, 1);

        // Files are cleaned up when ctx is dropped
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn insert_failure_expected() {
        let ctx = TestContext::memory();

        let coll = ctx
            .collection("constrained")
            .dimension(3)
            .with_payload("required_field TEXT NOT NULL")
            .create();

        // This should fail due to NOT NULL constraint
        let result = coll
            .insert(1)
            .vector(vec![1.0, 2.0, 3.0])
            .payload(&format!(
                "INSERT INTO {} (rowid, required_field) VALUES (?1, NULL)",
                coll.name
            ))
            .execute();

        assert_err!(result);
    }

    #[test]
    fn duplicate_insert() {
        let ctx = TestContext::memory();
        let coll = ctx.quick_collection("dup_test");

        // First insert succeeds
        coll.insert(1).vector(vec![1.0, 0.0, 0.0]).execute_ok();

        // Duplicate should fail
        let result = coll.insert(1).vector(vec![2.0, 0.0, 0.0]).execute();
        assert_err!(result);
    }
}

// ============================================================================
// Advanced Assertions
// ============================================================================

mod advanced_assertions {
    use super::*;

    #[test]
    fn result_content_assertions() {
        let ctx = TestContext::memory();
        let coll = fixtures::populated_users(&ctx);

        let results = coll
            .search(vec![1.0, 0.0, 0.0])
            .top_k(10)
            .payload(&format!("SELECT rowid, name, role FROM {}", coll.name))
            .execute();

        // Check various assertions
        assert_not_empty!(results);
        assert_at_least!(results, 1);
        assert_at_most!(results, 10);

        // Check specific content
        let names = get_field_values(&results, "name");
        assert!(names.contains(&"Alice".to_string()));

        // Check rowids
        let rowids = get_rowids(&results);
        assert!(rowids.contains(&1));
    }

    #[test]
    fn collection_state_assertions() {
        let ctx = TestContext::memory();
        let coll = ctx.quick_collection("state_test");

        assert_collection_empty!(coll);

        coll.insert_vector(1, vec![1.0, 0.0, 0.0]);

        assert_collection_not_empty!(coll);
        assert_collection_count!(coll, 1);
    }
}

// ============================================================================
// Raw SQL Access
// ============================================================================

mod raw_sql {
    use super::*;

    #[test]
    fn execute_raw_sql() {
        let ctx = TestContext::memory();

        // Create a regular table alongside vector collections
        ctx.execute_sql("CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT)")
            .expect("create table");

        ctx.execute_sql("INSERT INTO settings (key, value) VALUES ('version', '1.0')")
            .expect("insert");

        // Query with custom function
        let values: Vec<String> = ctx
            .query_sql("SELECT value FROM settings WHERE key = 'version'", |row| {
                row.get(0)
            })
            .expect("query");

        assert_eq!(values, vec!["1.0".to_string()]);
    }
}

// ============================================================================
// Distance Function Tests
// ============================================================================

mod distance_functions {
    use super::*;

    #[test]
    fn cosine_distance() {
        let ctx = TestContext::memory();
        let coll = ctx.collection("cosine").dimension(3).cosine().create();

        coll.insert_vector(1, vec![1.0, 0.0, 0.0]);
        coll.insert_vector(2, vec![0.0, 1.0, 0.0]);

        // Searching for [1, 0, 0] should find vector 1 first
        let results = coll.search(vec![0.9, 0.1, 0.0]).top_k(2).execute();
        assert_result_count!(results, 2);
    }

    #[test]
    fn l2_distance() {
        let ctx = TestContext::memory();
        let coll = ctx.collection("l2").dimension(3).l2().create();

        coll.insert_vector(1, vec![0.0, 0.0, 0.0]);
        coll.insert_vector(2, vec![1.0, 1.0, 1.0]);

        // Searching near origin should find vector 1 first
        let results = coll.search(vec![0.1, 0.1, 0.1]).top_k(2).execute();
        assert_result_count!(results, 2);
    }
}
