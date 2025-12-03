use vector_xlite::{VectorXLite, types::*};

pub fn run_simple_example(vlite: &VectorXLite) {
    // Check if collection already exists before creating
    let collection_name = "person";
    let exists = vlite.collection_exists(collection_name).unwrap();

    if exists {
        println!("Collection '{}' already exists, skipping creation.", collection_name);
    } else {
        println!("Collection '{}' does not exist, creating...", collection_name);
    }

    let config = CollectionConfigBuilder::default()
        .collection_name(collection_name)
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .payload_table_schema("create table person (rowid integer primary key, name text)")
        .build()
        .unwrap();

    match vlite.create_collection(config) {
        Ok(_) => {
            println!("Collection '{}' created successfully.", collection_name);
            let points = vec![
                InsertPoint::builder()
                    .collection_name("person")
                    .id(1)
                    .vector(vec![1.0, 2.0, 3.0, 4.0])
                    .payload_insert_query("insert into person(rowid, name) values (?1, 'Alice')")
                    .build()
                    .unwrap(),
                InsertPoint::builder()
                    .collection_name("person")
                    .id(2)
                    .vector(vec![4.0, 5.0, 6.0, 4.0])
                    .payload_insert_query("insert into person(name, rowid) values ('Bob', ?1)")
                    .build()
                    .unwrap(),
                InsertPoint::builder()
                    .collection_name("person")
                    .id(3)
                    .vector(vec![7.0, 8.0, 9.0, 4.0])
                    .payload_insert_query("insert into person values ('Charlie')")
                    .build()
                    .unwrap(),
                InsertPoint::builder()
                    .collection_name("person")
                    .id(5)
                    .vector(vec![17.0, 11.0, 9.0, 4.0])
                    .payload_insert_query("insert into person(name) values ('Charlie')")
                    .build()
                    .unwrap(),
            ];

            for point in points {
                vlite.insert(point).unwrap();
            }

            println!("Inserted points into 'person' collection.");

            let search_point = SearchPoint::builder()
                .collection_name("person")
                .vector(vec![7.0, 8.0, 9.0, 2.0])
                .top_k(10)
                .payload_search_query("select * from person")
                .build()
                .unwrap();

            let results = vlite.search(search_point).unwrap();

            println!("Search results: {:?}", results);

            // Verify collection exists after operations
            let still_exists = vlite.collection_exists(collection_name).unwrap();
            println!("Collection '{}' exists after operations: {}", collection_name, still_exists);
        }
        Err(e) => {
            println!("Error creating collection: {:?}", e);
            // Even if creation fails, check if collection partially exists
            let exists_on_error = vlite.collection_exists(collection_name).unwrap();
            println!("Collection '{}' exists after error: {}", collection_name, exists_on_error);
        }
    }
}
