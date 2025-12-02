use vector_xlite::{VectorXLite, types::*};

pub fn run_simple_example(vlite: &VectorXLite) {
    let config = CollectionConfigBuilder::default()
        .collection_name("person")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .payload_table_schema("create table person (rowid integer primary key, name text)")
        .build()
        .unwrap();

    match vlite.create_collection(config) {
        Ok(_) => {
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
        }
        Err(e) => println!("Error creating collection: {:?}", e),
    }
}
