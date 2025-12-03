//! Tests for builder patterns: CollectionConfigBuilder, InsertPointBuilder, SearchPointBuilder
//!
//! These tests verify:
//! - Required field validation
//! - Default value behavior
//! - Edge cases in builder inputs

use vector_xlite::types::*;

// ============================================================================
// CollectionConfigBuilder Tests
// ============================================================================

mod collection_config_builder {
    use super::*;

    #[test]
    fn build_succeeds_with_minimum_required_fields() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test_collection")
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.collection_name, "test_collection");
    }

    #[test]
    fn build_fails_without_collection_name() {
        let result = CollectionConfigBuilder::default()
            .vector_dimension(128)
            .build();

        assert!(result.is_err());
        match result {
            Err(msg) => assert_eq!(msg, "Collection_name must be provided."),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn default_dimension_is_3() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .build()
            .unwrap();

        assert_eq!(config.dimension, 3);
    }

    #[test]
    fn default_distance_is_cosine() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .build()
            .unwrap();

        assert_eq!(config.distance.as_str(), "cosine");
    }

    #[test]
    fn default_max_elements_is_100000() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .build()
            .unwrap();

        assert_eq!(config.max_elements, 100000);
    }

    #[test]
    fn custom_dimension_is_set() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .vector_dimension(512)
            .build()
            .unwrap();

        assert_eq!(config.dimension, 512);
    }

    #[test]
    fn all_distance_functions_work() {
        for (distance, expected_str) in [
            (DistanceFunction::Cosine, "cosine"),
            (DistanceFunction::L2, "l2"),
            (DistanceFunction::IP, "ip"),
        ] {
            let config = CollectionConfigBuilder::default()
                .collection_name("test")
                .distance(distance)
                .build()
                .unwrap();

            assert_eq!(config.distance.as_str(), expected_str);
        }
    }

    #[test]
    fn payload_table_schema_is_set() {
        let schema = "CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)";
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .payload_table_schema(schema)
            .build()
            .unwrap();

        assert_eq!(config.payload_table_schema, Some(schema.to_string()));
    }

    #[test]
    fn default_payload_schema_generated_when_none_provided() {
        let config = CollectionConfigBuilder::default()
            .collection_name("my_collection")
            .build()
            .unwrap();

        // When no schema provided, a default is generated
        assert!(config.payload_table_schema.is_some());
        let schema = config.payload_table_schema.unwrap();
        assert!(schema.contains("my_collection"));
        assert!(schema.to_lowercase().contains("create table"));
    }

    #[test]
    fn index_file_path_is_set() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .index_file_path("/path/to/index.bin")
            .build()
            .unwrap();

        assert_eq!(
            config.index_file_path,
            Some("/path/to/index.bin".to_string())
        );
    }

    #[test]
    fn max_elements_is_set() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .max_elements(500000)
            .build()
            .unwrap();

        assert_eq!(config.max_elements, 500000);
    }

    #[test]
    fn builder_accepts_string_types() {
        // Test that Into<String> works for various string types
        let name = String::from("owned_string");
        let config = CollectionConfigBuilder::default()
            .collection_name(name)
            .build()
            .unwrap();

        assert_eq!(config.collection_name, "owned_string");
    }

    #[test]
    fn zero_dimension_is_accepted() {
        // Note: This might be a design issue - 0 dimension doesn't make sense
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .vector_dimension(0)
            .build()
            .unwrap();

        assert_eq!(config.dimension, 0);
    }

    #[test]
    fn max_u16_dimension_is_accepted() {
        let config = CollectionConfigBuilder::default()
            .collection_name("test")
            .vector_dimension(u16::MAX)
            .build()
            .unwrap();

        assert_eq!(config.dimension, u16::MAX);
    }
}

// ============================================================================
// InsertPointBuilder Tests
// ============================================================================

mod insert_point_builder {
    use super::*;

    #[test]
    fn build_succeeds_with_required_fields() {
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![1.0, 2.0, 3.0])
            .build();

        assert!(point.is_ok());
        let point = point.unwrap();
        assert_eq!(point.collection_name, "test");
        assert_eq!(point.vector, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn build_fails_without_collection_name() {
        let result = InsertPoint::builder()
            .vector(vec![1.0, 2.0, 3.0])
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Collection_name must be provided.");
    }

    #[test]
    fn build_fails_without_vector() {
        let result = InsertPoint::builder().collection_name("test").build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vector must be provided.");
    }

    #[test]
    fn id_is_optional() {
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .build()
            .unwrap();

        assert!(point.id.is_none());
    }

    #[test]
    fn id_is_set_when_provided() {
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .id(42)
            .build()
            .unwrap();

        assert_eq!(point.id, Some(42));
    }

    #[test]
    fn payload_insert_query_is_optional() {
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .build()
            .unwrap();

        assert!(point.payload_insert_query.is_none());
    }

    #[test]
    fn payload_insert_query_is_set_when_provided() {
        let query = "INSERT INTO test(rowid, data) VALUES (?1, 'value')";
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .payload_insert_query(query)
            .build()
            .unwrap();

        assert_eq!(point.payload_insert_query, Some(query.to_string()));
    }

    #[test]
    fn empty_vector_is_accepted() {
        // Note: This might be a design issue - empty vectors don't make sense
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![])
            .build()
            .unwrap();

        assert!(point.vector.is_empty());
    }

    #[test]
    fn large_vector_is_accepted() {
        let large_vec: Vec<f32> = (0..10000).map(|i| i as f32).collect();
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(large_vec.clone())
            .build()
            .unwrap();

        assert_eq!(point.vector.len(), 10000);
    }

    #[test]
    fn special_float_values_in_vector() {
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -0.0])
            .build()
            .unwrap();

        assert_eq!(point.vector.len(), 5);
        assert!(point.vector[0].is_nan());
        assert!(point.vector[1].is_infinite());
    }

    #[test]
    fn max_u64_id_is_accepted() {
        let point = InsertPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .id(u64::MAX)
            .build()
            .unwrap();

        assert_eq!(point.id, Some(u64::MAX));
    }
}

// ============================================================================
// SearchPointBuilder Tests
// ============================================================================

mod search_point_builder {
    use super::*;

    #[test]
    fn build_succeeds_with_required_fields() {
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0, 2.0, 3.0])
            .build();

        assert!(search.is_ok());
        let search = search.unwrap();
        assert_eq!(search.collection_name, "test");
        assert_eq!(search.vector, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn build_fails_without_collection_name() {
        let result = SearchPoint::builder()
            .vector(vec![1.0, 2.0, 3.0])
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Collection_name must be provided.");
    }

    #[test]
    fn build_fails_without_vector() {
        let result = SearchPoint::builder().collection_name("test").build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vector must be provided.");
    }

    #[test]
    fn default_top_k_is_10() {
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .build()
            .unwrap();

        assert_eq!(search.top_k, 10);
    }

    #[test]
    fn custom_top_k_is_set() {
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .top_k(100)
            .build()
            .unwrap();

        assert_eq!(search.top_k, 100);
    }

    #[test]
    fn zero_top_k_fails() {
        let result = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .top_k(0)
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "top_k must be greater than 0.");
    }

    #[test]
    fn negative_top_k_fails() {
        let result = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .top_k(-5)
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "top_k must be greater than 0.");
    }

    #[test]
    fn payload_search_query_is_optional() {
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .build()
            .unwrap();

        assert!(search.payload_search_query.is_none());
    }

    #[test]
    fn payload_search_query_is_set_when_provided() {
        let query = "SELECT * FROM test WHERE rating > 4";
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .payload_search_query(query)
            .build()
            .unwrap();

        assert_eq!(search.payload_search_query, Some(query.to_string()));
    }

    #[test]
    fn very_large_top_k_is_accepted() {
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![1.0])
            .top_k(i64::MAX)
            .build()
            .unwrap();

        assert_eq!(search.top_k, i64::MAX);
    }

    #[test]
    fn empty_vector_search() {
        let search = SearchPoint::builder()
            .collection_name("test")
            .vector(vec![])
            .build()
            .unwrap();

        assert!(search.vector.is_empty());
    }
}

// ============================================================================
// DistanceFunction Tests
// ============================================================================

mod distance_function {
    use super::*;

    #[test]
    fn l2_as_str() {
        assert_eq!(DistanceFunction::L2.as_str(), "l2");
    }

    #[test]
    fn cosine_as_str() {
        assert_eq!(DistanceFunction::Cosine.as_str(), "cosine");
    }

    #[test]
    fn ip_as_str() {
        assert_eq!(DistanceFunction::IP.as_str(), "ip");
    }
}
