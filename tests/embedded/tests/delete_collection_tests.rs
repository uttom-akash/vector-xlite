
//! Tests for delete_collection method in VectorXLite
//
//! These tests verify:
//! - Deleting an existing collection works as expected
//! - Deleting a non-existent collection returns an error

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
fn test_delete_collection() {
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

    assert!(
        vlite.collection_exists("person").unwrap(),
        "Collection should exist before deletion"
    );

    let delete_collection = DeleteCollection::builder()
        .collection_name("person")
        .build()
        .expect("Builder should create delete collection.");

    vlite
        .delete_collection(delete_collection)
        .expect("delete should be successful");

    assert!(
        !vlite.collection_exists("person").unwrap(),
        "Collection should not exist after deletion"
    );
}

#[test]
fn test_delete_non_existent_collection() {
    let (vlite, _) = setup_vlite();

    let delete_collection = DeleteCollection::builder()
        .collection_name("non_existent_collection")
        .build()
        .expect("Builder should create delete collection.");

    let result = vlite.delete_collection(delete_collection);

    assert!(result.is_err(), "Should return an error for non-existent collection");
}
