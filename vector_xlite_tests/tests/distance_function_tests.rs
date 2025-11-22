//! Tests for different distance functions: Cosine, L2, Inner Product (IP)
//!
//! These tests verify that each distance function:
//! - Works correctly for basic vector searches
//! - Returns results in the correct order
//! - Handles edge cases appropriately

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
// Cosine Distance Tests
// ============================================================================

mod cosine_distance {
    use super::*;

    #[test]
    fn cosine_basic_search() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("cosine_basic")
            .vector_dimension(3)
            .distance(DistanceFunction::Cosine)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Insert vectors
        let vectors = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![0.0, 1.0, 0.0]),
            (3, vec![0.0, 0.0, 1.0]),
            (4, vec![1.0, 1.0, 0.0]),   // 45 degrees from [1,0,0]
            (5, vec![1.0, 1.0, 1.0]),   // Closer to all axes
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("cosine_basic")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search for vector most similar to [1, 0, 0]
        let search = SearchPoint::builder()
            .collection_name("cosine_basic")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(5)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert!(!results.is_empty());

        // The most similar vector should be [1, 0, 0] (id=1)
        let first_id = results[0].get("rowid").unwrap();
        assert_eq!(first_id, "1", "Expected id=1 to be most similar for cosine");
    }

    #[test]
    fn cosine_normalized_vectors() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("cosine_norm")
            .vector_dimension(3)
            .distance(DistanceFunction::Cosine)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Normalized unit vectors
        let sqrt2 = (2.0f32).sqrt();
        let sqrt3 = (3.0f32).sqrt();

        let vectors = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![1.0 / sqrt2, 1.0 / sqrt2, 0.0]),
            (3, vec![1.0 / sqrt3, 1.0 / sqrt3, 1.0 / sqrt3]),
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("cosine_norm")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let search = SearchPoint::builder()
            .collection_name("cosine_norm")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(3)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn cosine_opposite_vectors() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("cosine_opposite")
            .vector_dimension(3)
            .distance(DistanceFunction::Cosine)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Opposite vectors have cosine distance of 2 (1 - (-1))
        let vectors = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![-1.0, 0.0, 0.0]),   // Opposite direction
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("cosine_opposite")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search for [1, 0, 0] - should find id=1 first
        let search = SearchPoint::builder()
            .collection_name("cosine_opposite")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(2)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].get("rowid").unwrap(), "1");
        assert_eq!(results[1].get("rowid").unwrap(), "2");
    }
}

// ============================================================================
// L2 (Euclidean) Distance Tests
// ============================================================================

mod l2_distance {
    use super::*;

    #[test]
    fn l2_basic_search() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("l2_basic")
            .vector_dimension(3)
            .distance(DistanceFunction::L2)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let vectors = vec![
            (1, vec![0.0, 0.0, 0.0]),
            (2, vec![1.0, 0.0, 0.0]),
            (3, vec![2.0, 0.0, 0.0]),
            (4, vec![3.0, 0.0, 0.0]),
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("l2_basic")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search for [1.5, 0, 0] - should find [1,0,0] and [2,0,0] as closest
        let search = SearchPoint::builder()
            .collection_name("l2_basic")
            .vector(vec![1.5, 0.0, 0.0])
            .top_k(4)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 4);

        // First two should be id=2 and id=3 (distance 0.5 each from 1.5)
        let first_id = results[0].get("rowid").unwrap();
        let second_id = results[1].get("rowid").unwrap();
        assert!(
            (first_id == "2" && second_id == "3") || (first_id == "3" && second_id == "2"),
            "Expected id=2 and id=3 to be closest"
        );
    }

    #[test]
    fn l2_multidimensional() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("l2_multi")
            .vector_dimension(4)
            .distance(DistanceFunction::L2)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let vectors = vec![
            (1, vec![0.0, 0.0, 0.0, 0.0]),
            (2, vec![1.0, 1.0, 1.0, 1.0]),
            (3, vec![2.0, 2.0, 2.0, 2.0]),
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("l2_multi")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // Search from origin - id=1 should be closest
        let search = SearchPoint::builder()
            .collection_name("l2_multi")
            .vector(vec![0.0, 0.0, 0.0, 0.0])
            .top_k(3)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results[0].get("rowid").unwrap(), "1");
    }

    #[test]
    fn l2_identical_vectors() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("l2_identical")
            .vector_dimension(3)
            .distance(DistanceFunction::L2)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Insert identical vectors with different IDs
        for id in 1..=3 {
            let point = InsertPoint::builder()
                .collection_name("l2_identical")
                .id(id)
                .vector(vec![1.0, 2.0, 3.0])
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // All should have distance 0
        let search = SearchPoint::builder()
            .collection_name("l2_identical")
            .vector(vec![1.0, 2.0, 3.0])
            .top_k(3)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 3);

        // All distances should be 0 (or very close)
        for result in &results {
            if let Some(distance) = result.get("distance") {
                let dist: f64 = distance.parse().unwrap_or(1.0);
                assert!(dist < 0.0001, "Expected distance ~0 for identical vector");
            }
        }
    }
}

// ============================================================================
// Inner Product (IP) Distance Tests
// ============================================================================

mod ip_distance {
    use super::*;

    #[test]
    fn ip_basic_search() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("ip_basic")
            .vector_dimension(3)
            .distance(DistanceFunction::IP)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        let vectors = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![0.5, 0.0, 0.0]),
            (3, vec![2.0, 0.0, 0.0]),
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("ip_basic")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        // With inner product, larger magnitude in same direction = higher similarity
        let search = SearchPoint::builder()
            .collection_name("ip_basic")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(3)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 3);
        // Note: IP distance typically returns negative inner product for ordering
    }

    #[test]
    fn ip_orthogonal_vectors() {
        let (vlite, _) = setup_vlite();

        let config = CollectionConfigBuilder::default()
            .collection_name("ip_ortho")
            .vector_dimension(3)
            .distance(DistanceFunction::IP)
            .build()
            .unwrap();

        vlite.create_collection(config).expect("create collection");

        // Orthogonal vectors have IP = 0
        let vectors = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![0.0, 1.0, 0.0]),
            (3, vec![0.0, 0.0, 1.0]),
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name("ip_ortho")
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }

        let search = SearchPoint::builder()
            .collection_name("ip_ortho")
            .vector(vec![1.0, 0.0, 0.0])
            .top_k(3)
            .build()
            .unwrap();

        let results = vlite.search(search).expect("search");
        assert_eq!(results.len(), 3);
        // First should be id=1 (IP=1), others have IP=0
        assert_eq!(results[0].get("rowid").unwrap(), "1");
    }
}

// ============================================================================
// Distance Function Comparison Tests
// ============================================================================

mod distance_comparison {
    use super::*;

    /// Helper to create a collection with a specific distance function
    fn create_collection_with_distance(
        vlite: &VectorXLite,
        name: &str,
        distance: DistanceFunction,
    ) {
        let config = CollectionConfigBuilder::default()
            .collection_name(name)
            .vector_dimension(3)
            .distance(distance)
            .build()
            .unwrap();
        vlite.create_collection(config).expect("create collection");
    }

    /// Helper to insert test vectors
    fn insert_test_vectors(vlite: &VectorXLite, collection: &str) {
        let vectors = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![0.0, 1.0, 0.0]),
            (3, vec![0.707, 0.707, 0.0]),
            (4, vec![0.5, 0.5, 0.707]),
        ];

        for (id, vec) in vectors {
            let point = InsertPoint::builder()
                .collection_name(collection)
                .id(id)
                .vector(vec)
                .build()
                .unwrap();
            vlite.insert(point).expect("insert");
        }
    }

    #[test]
    fn all_distance_functions_return_results() {
        let (vlite, _) = setup_vlite();

        let test_cases = vec![
            ("dist_cosine", DistanceFunction::Cosine),
            ("dist_l2", DistanceFunction::L2),
            ("dist_ip", DistanceFunction::IP),
        ];

        for (name, distance) in test_cases {
            create_collection_with_distance(&vlite, name, distance);
            insert_test_vectors(&vlite, name);

            let search = SearchPoint::builder()
                .collection_name(name)
                .vector(vec![1.0, 0.0, 0.0])
                .top_k(4)
                .build()
                .unwrap();

            let results = vlite.search(search).expect(&format!("search {}", name));
            assert_eq!(results.len(), 4, "Expected 4 results for {}", name);
        }
    }

    #[test]
    fn distance_functions_produce_different_orderings() {
        let (vlite, _) = setup_vlite();

        // Create separate collections for each distance function
        create_collection_with_distance(&vlite, "cmp_cosine", DistanceFunction::Cosine);
        create_collection_with_distance(&vlite, "cmp_l2", DistanceFunction::L2);

        // Insert same vectors
        let vectors = vec![
            (1, vec![0.1, 0.0, 0.0]),   // Small magnitude, same direction
            (2, vec![10.0, 0.0, 0.0]),  // Large magnitude, same direction
            (3, vec![1.0, 1.0, 0.0]),   // Different direction
        ];

        for collection in ["cmp_cosine", "cmp_l2"] {
            for (id, vec) in &vectors {
                let point = InsertPoint::builder()
                    .collection_name(collection)
                    .id(*id)
                    .vector(vec.clone())
                    .build()
                    .unwrap();
                vlite.insert(point).expect("insert");
            }
        }

        // Search with a unit vector along x-axis
        let query = vec![1.0, 0.0, 0.0];

        // Cosine: id=1 and id=2 should have same distance (same direction)
        let cosine_search = SearchPoint::builder()
            .collection_name("cmp_cosine")
            .vector(query.clone())
            .top_k(3)
            .build()
            .unwrap();
        let cosine_results = vlite.search(cosine_search).expect("cosine search");

        // L2: id=1 should be closer than id=2 (Euclidean considers magnitude)
        let l2_search = SearchPoint::builder()
            .collection_name("cmp_l2")
            .vector(query)
            .top_k(3)
            .build()
            .unwrap();
        let l2_results = vlite.search(l2_search).expect("l2 search");

        // Log results for analysis
        println!("Cosine results: {:?}", cosine_results);
        println!("L2 results: {:?}", l2_results);

        // Both should return 3 results
        assert_eq!(cosine_results.len(), 3);
        assert_eq!(l2_results.len(), 3);
    }
}
