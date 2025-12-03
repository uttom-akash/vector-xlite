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

#[test]
fn delete_search_return_no_deleted_entry() {
    let (vlite, _) = setup_vlite();

    let config = CollectionConfigBuilder::default()
        .collection_name("person")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .payload_table_schema("create table person (rowid integer primary key, name text)")
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection should be created");

    let point = InsertPoint::builder()
        .collection_name("person")
        .id(1)
        .vector(vec![1.0, 2.0, 3.0, 4.0])
        .payload_insert_query("insert into person(rowid, name) values (?1, 'Alice')")
        .build()
        .expect("Builder should create insert point.");

    vlite
        .insert(point)
        .expect("insert should be successful.");

    let search_point = SearchPoint::builder()
        .collection_name("person")
        .vector(vec![7.0, 8.0, 9.0, 2.0])
        .top_k(10)
        .payload_search_query("select * from person")
        .build()
        .expect("Builder should create search point.");

    let search_before_delete = vlite.search(search_point.clone()).unwrap();

    dbg!(&search_before_delete);

    assert!(
        search_before_delete.iter().any(|hm| hm["rowid"] == stringify!(1)),
        "Vector entry should exist before deletion"
    );

    let delete_point = DeletePoint::builder()
        .collection_name("person")
        .id(1)
        .build()
        .expect("Builder should create delete point.");

    vlite
        .delete(delete_point)
        .expect("delete should be successful");

    let search_after_delete = vlite.search(search_point).unwrap();

    assert!(
        !search_after_delete.iter().any(|hm| hm["rowid"] == stringify!(1)),
        "Vector entry should not exist after deletion"
    );
}

#[test]
fn delete_search_return_no_deleted_entry_without_payload() {
    let (vlite, _) = setup_vlite();

    let config = CollectionConfigBuilder::default()
        .collection_name("vectors")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection should be created");

    let point = InsertPoint::builder()
        .collection_name("vectors")
        .id(1)
        .vector(vec![1.0, 2.0, 3.0, 4.0])
        .build()
        .expect("Builder should create insert point.");

    vlite
        .insert(point)
        .expect("insert should be successful.");

    let search_point = SearchPoint::builder()
        .collection_name("vectors")
        .vector(vec![7.0, 8.0, 9.0, 2.0])
        .top_k(10)
        .build()
        .expect("Builder should create search point.");

    let search_before_delete = vlite.search(search_point.clone()).unwrap();

    dbg!(&search_before_delete);

    assert_eq!(
        search_before_delete.len(),
        1,
        "Should find exactly one vector before deletion"
    );

    let delete_point = DeletePoint::builder()
        .collection_name("vectors")
        .id(1)
        .build()
        .expect("Builder should create delete point.");

    vlite
        .delete(delete_point)
        .expect("delete should be successful");

    let search_after_delete = vlite.search(search_point).unwrap();

    assert_eq!(
        search_after_delete.len(),
        0,
        "Should find no vectors after deletion"
    );
}

#[test]
fn delete_multiple_entries_without_payload() {
    let (vlite, _) = setup_vlite();

    let config = CollectionConfigBuilder::default()
        .collection_name("vectors")
        .distance(DistanceFunction::L2)
        .vector_dimension(3)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection should be created");

    // Insert multiple points
    for id in 1..=5 {
        let point = InsertPoint::builder()
            .collection_name("vectors")
            .id(id)
            .vector(vec![id as f32, id as f32 * 2.0, id as f32 * 3.0])
            .build()
            .expect("Builder should create insert point.");

        vlite
            .insert(point)
            .expect("insert should be successful.");
    }

    let search_point = SearchPoint::builder()
        .collection_name("vectors")
        .vector(vec![3.0, 6.0, 9.0])
        .top_k(10)
        .build()
        .expect("Builder should create search point.");

    let search_before_delete = vlite.search(search_point.clone()).unwrap();

    assert_eq!(
        search_before_delete.len(),
        5,
        "Should find all 5 vectors before deletion"
    );

    // Delete entries with id 2 and 4
    for id in [2, 4] {
        let delete_point = DeletePoint::builder()
            .collection_name("vectors")
            .id(id)
            .build()
            .expect("Builder should create delete point.");

        vlite
            .delete(delete_point)
            .expect("delete should be successful");
    }

    let search_after_delete = vlite.search(search_point).unwrap();

    assert_eq!(
        search_after_delete.len(),
        3,
        "Should find 3 vectors after deleting 2"
    );
}

#[test]
fn search_empty_collection_without_payload() {
    let (vlite, _) = setup_vlite();

    let config = CollectionConfigBuilder::default()
        .collection_name("empty_vectors")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection should be created");

    let search_point = SearchPoint::builder()
        .collection_name("empty_vectors")
        .vector(vec![1.0, 2.0, 3.0, 4.0])
        .top_k(10)
        .build()
        .expect("Builder should create search point.");

    let search_results = vlite.search(search_point).unwrap();

    assert_eq!(
        search_results.len(),
        0,
        "Should return empty results for empty collection"
    );
}

#[test]
fn insert_and_search_without_payload() {
    let (vlite, _) = setup_vlite();

    let config = CollectionConfigBuilder::default()
        .collection_name("simple_vectors")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(2)
        .build()
        .unwrap();

    vlite
        .create_collection(config)
        .expect("collection should be created");

    let point = InsertPoint::builder()
        .collection_name("simple_vectors")
        .id(42)
        .vector(vec![0.5, 0.8])
        .build()
        .expect("Builder should create insert point.");

    vlite
        .insert(point)
        .expect("insert should be successful.");

    let search_point = SearchPoint::builder()
        .collection_name("simple_vectors")
        .vector(vec![0.6, 0.7])
        .top_k(5)
        .build()
        .expect("Builder should create search point.");

    let search_results = vlite.search(search_point).unwrap();

    assert_eq!(
        search_results.len(),
        1,
        "Should find the inserted vector"
    );
}