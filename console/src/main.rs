use sqlite_vectorx::{CollectionConfigBuilder, DistanceFunction, VectorXLite};
use rusqlite::Connection;

fn main() {
    let sqlite_connection = Connection::open_in_memory().unwrap();

    let vs = VectorXLite::new(sqlite_connection).unwrap();

    let config = CollectionConfigBuilder::default()
        .distance(DistanceFunction::L2)
        .payload_table_schema("create table person (rowid integer primary key, name text)")
        .build(); 
    
    match vs.create_collection(config) {
        Ok(_) => {
            vs.insert_vector(
                1,
                &vec![1.0, 2.0, 3.0],
                "insert into person(rowid, name) values (?1, 'Alice')",
            )
            .unwrap();
            vs.insert_vector(
                2,
                &vec![4.0, 5.0, 6.0],
                "insert into person(name, rowid) values ('Bob', ?1)",
            )
            .unwrap();
            vs.insert_vector(
                3,
                &vec![7.0, 8.0, 9.0],
                "insert into person values ('Charlie')",
            )
            .unwrap();

            vs.insert_vector(
                5,
                &vec![17.0, 11.0, 9.0],
                "insert into person(name) values ('Charlie')",
            )
            .unwrap();

            let results = vs
                .search_vectors(&vec![7.0, 8.0, 9.0], 10, "select * from person")
                .unwrap();

            println!("Search results: {:?}", results);
        }
        Err(e) => println!("Error creating collection: {:?}", e),
    }
}
