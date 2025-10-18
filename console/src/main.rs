use sqlite_vectorx::VectorXSqlite;

fn main() {
    // println!("Hello, world!");

    // let nb_elem = 16;
    // let max_nb_connection = 24;
    // let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
    // let ef_construction = 400;

    // let index = VectorIndexEngineImpl::new(max_nb_connection, nb_elem, nb_layer, ef_construction);

    // index.insert(Point {
    //     id: 1,
    //     vector: vec![1.0, 2.0, 3.0],
    // });
    // index.insert(Point {
    //     id: 2,
    //     vector: vec![4.0, 5.0, 6.0],
    // });
    // index.insert(Point {
    //     id: 3,
    //     vector: vec![7.0, 8.0, 9.0],
    // });

    // let results = index.search(vec![7.0, 8.0, 9.0], 2);
    // println!("The result of adding 10 and 15 is: {:?}", results);

    let vs = VectorXSqlite::new().unwrap();
    match vs.create_collection(
        "person",
        "create table person (rowid integer primary key, name text)",
    ) {
        Ok(_) => {
            vs.insert_vector(
                "person",
                1,
                &vec![1.0, 2.0, 3.0],
                "insert into person(rowid, name) values (?1, 'Alice')",
            )
            .unwrap();
            vs.insert_vector(
                "person",
                2,
                &vec![4.0, 5.0, 6.0],
                "insert into person(rowid, name) values (?1, 'Bob')",
            )
            .unwrap();
            vs.insert_vector(
                "person",
                3,
                &vec![7.0, 8.0, 9.0],
                "insert into person(rowid, name) values (?1, 'Charlie')",
            )
            .unwrap();

            let results = vs
                .search_vectors(
                    "person",
                    &vec![7.0, 8.0, 9.0],
                    1,
                    "select * from person as p where p.name in ('Charlie')",
                )
                .unwrap();

            println!("Search results: {:?}", results);
        }
        Err(e) => println!("Error creating collection: {:?}", e),
    }
}
