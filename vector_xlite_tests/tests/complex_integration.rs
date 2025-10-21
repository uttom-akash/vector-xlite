use std::rc::Rc;

use rusqlite::Connection;
use vector_xlite::{VectorXLite, types::*};

/// Integration tests based on the complex_example scenario.
/// These tests exercise collection creation, payload inserts and a hybrid
/// vector + SQL payload-aware search. They avoid asserting on exact
/// return types — tests check for successful execution and for expected
/// content in the results.
fn setup_vlite() -> (VectorXLite, Rc<Connection>) {
    let conn = Rc::new(Connection::open_in_memory().expect("open in-memory sqlite"));
    let vlite = VectorXLite::new(Rc::clone(&conn)).expect("create VectorXLite");
    (vlite, conn)
}

fn insert_authors(conn: &Connection) {
    let create_authors_table = r#"
    create table authors (
        id integer primary key,
        name text not null,
        bio text
    );
    "#;

    conn.execute(create_authors_table, [])
        .expect("create authors table");

    let author_inserts = vec![
        "insert into authors(id, name, bio) values (1, 'Alice', 'Writer of whimsical fantasy worlds')",
        "insert into authors(id, name, bio) values (2, 'Bob', 'Short story enthusiast and poet')",
        "insert into authors(id, name, bio) values (3, 'Carol', 'Sci-fi novelist exploring deep space themes')",
    ];

    for q in author_inserts {
        conn.execute(q, []).expect("insert author");
    }
}

fn prepare_and_insert_stories(vlite: &VectorXLite) {
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

    vlite
        .create_collection(story_collection_config)
        .expect("create collection");

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

    for p in points {
        vlite.insert(p).expect("insert point");
    }
}

#[test]
fn test_advanced_story_search_filtered() {
    let (vlite, conn) = setup_vlite();
    insert_authors(&conn);
    prepare_and_insert_stories(&vlite);

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

    let results = vlite.search(search_point).expect("search executed");
    // Expect at least one result and that the highest-rated non-drama story is present.
    assert!(!results.is_empty(), "expected at least one search result");
    let results_str = format!("{:#?}", results);
    assert!(
        results_str.contains("Stars Beneath the Waves")
            || results_str.contains("Dreaming in Colors"),
        "expected known story titles in results, got: {}",
        results_str
    );
}

#[test]
fn test_advanced_story_search_broad() {
    let (vlite, conn) = setup_vlite();
    insert_authors(&conn);
    prepare_and_insert_stories(&vlite);

    let search_point = SearchPoint::builder()
        .collection_name("story_advanced")
        .vector(vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5])
        .top_k(10)
        .payload_search_query("select rowid, author_id, title, rating from story_advanced")
        .build()
        .unwrap();

    let results = vlite.search(search_point).expect("broad search executed");
    assert!(!results.is_empty(), "expected results from broad search");
    let results_str = format!("{:#?}", results);
    // All inserted ids should be represented somewhere in the results string
    assert!(
        results_str.contains("101") && results_str.contains("102") && results_str.contains("103"),
        "expected all inserted rowids in results, got: {}",
        results_str
    );
}
