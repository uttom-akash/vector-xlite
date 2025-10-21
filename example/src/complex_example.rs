use std::sync::Arc;

use rusqlite::Connection;
use vector_xlite::{VectorXLite, types::*};

pub fn run_complex_example(vlite: &VectorXLite, sqlite_conn: Arc<Connection>) {
    let create_authors_table = r#"
    create table authors (
            id integer primary key,
            name text not null,
            bio text
        );
        "#;

    sqlite_conn
        .execute(create_authors_table, [])
        .expect("Failed to create authors table");

    let author_inserts = vec![
        "insert into authors(id, name, bio) values (1, 'Alice', 'Writer of whimsical fantasy worlds')",
        "insert into authors(id, name, bio) values (2, 'Bob', 'Short story enthusiast and poet')",
        "insert into authors(id, name, bio) values (3, 'Carol', 'Sci-fi novelist exploring deep space themes')",
    ];

    for q in author_inserts {
        sqlite_conn.execute(q, []).unwrap();
    }

    let story_collection_config = CollectionConfigBuilder::default()
        .collection_name("story_advanced")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(8)
        .payload_table_schema(
            r#"
        create table story_advanced (
            rowid integer primary key,
            author_id integer,
            title text,
            content text,
            tags json,
            published_at text default (datetime('now')),
            rating real
        );
        "#,
        )
        .build()
        .unwrap();

    match vlite.create_collection(story_collection_config) {
        Ok(_) => {
            let points = vec![
            InsertPoint::builder()
                .collection_name("story_advanced")
                .id(101)
                .vector(vec![0.11, 0.22, 0.33, 0.44, 0.55, 0.66, 0.77, 0.88])
                .payload_insert_query(r#"
                    insert into story_advanced(rowid, author_id, title, content, tags, rating)
                    values (?1, 1, 'Dreaming in Colors', 'Once upon a vibrant night...', '["fantasy","dreams"]', 4.8)
                "#)
                .build()
                .unwrap(),
            InsertPoint::builder()
                .collection_name("story_advanced")
                .id(102)
                .vector(vec![0.9, 0.8, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6])
                .payload_insert_query(r#"
                    insert into story_advanced(rowid, author_id, title, content, tags, rating)
                    values (?1, 2, 'The Quiet Storm', 'Thunder rolled over the valley...', '["drama","short","weather"]', 4.2)
                "#)
                .build()
                .unwrap(),
            InsertPoint::builder()
                .collection_name("story_advanced")
                .id(103)
                .vector(vec![0.05, 0.25, 0.45, 0.65, 0.85, 0.15, 0.35, 0.55])
                .payload_insert_query(r#"
                    insert into story_advanced(rowid, author_id, title, content, tags, rating)
                    values (?1, 3, 'Stars Beneath the Waves', 'A galaxy reflected in the ocean depths...', '["sci-fi","ocean","space"]', 4.9)
                "#)
                .build()
                .unwrap(),
        ];

            for point in points {
                vlite.insert(point.clone()).unwrap();
            }

            println!("✅ Inserted complex story points into 'story_advanced' collection.");

            // Create a complex search point
            let search_point = SearchPoint::builder()
                .collection_name("story_advanced")
                .vector(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8])
                .top_k(5)
                .payload_search_query(
                    r#"
                select 
                    s.rowid, 
                    s.title, 
                    s.rating, 
                    a.name as author, 
                    s.tags, 
                    s.published_at
                from story_advanced s
                join authors a on a.id = s.author_id
                where s.rating > 4.0
                  and json_extract(s.tags, '$[0]') != 'drama'
                order by s.rating desc
                "#,
                )
                .build()
                .unwrap();

            // Perform the vector + SQL hybrid search
            let results = vlite.search(search_point).unwrap();

            println!("\n🚀 Advanced Story Search Results:\n{:#?}", results);
        }
        Err(e) => println!("❌ Error creating advanced story collection: {:?}", e),
    }
}
