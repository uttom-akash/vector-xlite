<h1 align="center" vertical-align="center">
  <img src="https://i.imgur.com/S3PJvXm.png" alt="vxlite logo" width="40"/>
  <span style="margin-bottom: 20px;">Vector Xlite</span>
</h1>

**VectorXLite** — A fast, lightweight vector search with payload support and SQL-based filtering.

Crate : https://crates.io/crates/vector_xlite

This demonstrates how to use the `vector_xlite` crate to:

- Create a collection with vector embeddings and optional payload data.  
- Insert and manage vectors along with associated metadata.  
- Perform fast vector similarity search (e.g., **Cosine**, **Dot**, or **L2** distance).  
- Filter and query payloads using standard **SQL** alongside vector search.

---

## 🧱 Step-by-Step Breakdown

### 1. Initialize Sqlite Connection Pool

*** Don't forget add this `.connection_customizer(SqliteConnectionCustomizer::new())`

```rust
// Create an r2d2 Sqlite connection manager in memory
let manager = SqliteConnectionManager::memory();

// Build a pool and attach a connection customizer that ensures
// the native extension (and other per-connection setup) run
let pool = Pool::builder()
    .max_size(15)
    .connection_customizer(SqliteConnectionCustomizer::new()) 
    .build(manager)
    .unwrap();

// Construct the VectorXLite API object from the pool
let vlite = VectorXLite::new(pool.clone()).unwrap();
```

### 2. Create the Collection

```rust
let config = CollectionConfigBuilder::default()
    .collection_name("person")
    .distance(DistanceFunction::Cosine)
    .vector_dimension(4)
    .payload_table_schema("create table person (rowid integer primary key, name text)")
    .build()
    .unwrap();

vlite.create_collection(config).unwrap();
```

This defines:

- collection_name — logical name for your vector data
- distance — similarity metric (Cosine, L2, or Dot)
- vector_dimension — length of the embedding vector
- payload_table_schema — SQL used to store associated metadata

### 2. Insert Vector Points

Each vector point includes an id, vector embedding, and an SQL payload insertion query.

```
let point = InsertPoint::builder()
    .collection_name("person")
    .id(1)
    .vector(vec![1.0, 2.0, 3.0, 4.0])
    .payload_insert_query("insert into person(rowid, name) values (?1, 'Alice')")
    .build()
    .unwrap();

vlite.insert(point).unwrap();
```

Use ?1 as a placeholder to bind the vector ID in your SQL statement.

### 3. Search for Similar Vectors

Perform a similarity search with a given vector and get top matches:

```rust
let search_point = SearchPoint::builder()
    .collection_name("person")
    .vector(vec![7.0, 8.0, 9.0, 2.0])
    .top_k(10)
    .payload_search_query("select * from person")
    .build()
    .unwrap();

let results = vlite.search(search_point).unwrap();
```

This fetches the top-K most similar vectors from the collection, along with their payloads.


## 🚀 Console Example — `vector_xlite`

A minimal Rust example showing how to use the `vector_xlite` crate via the included `example` binary.

The example opens an **in-memory SQLite** connection, registers the VectorXLite extension, creates a collection, inserts several vector points with payloads, and performs a vector search.

---

## 🧩 Prerequisites

- **Rust** (latest stable)
- **SQLite** (with extension loading enabled)

---

## ▶️ Running the Example

From the repository root:

```bash
cd example
cargo run
```

Or run the specific package directly:
``` bash
cargo run -p example
```

## 📘 Full Example

This example corresponds to the contents of src/main.rs inside the example crate:

```rust
use vector_xlite::{
    types::{SearchPoint, CollectionConfigBuilder, InsertPoint, DistanceFunction},
    VectorXLite,
    customizer::SqliteConnectionCustomizer
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;


fn main() {
    // Step 1: Open SQLite in memory
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder()
        .max_size(15)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .unwrap();

    let vlite = VectorXLite::new(pool.clone()).unwrap();

    // Step 3: Configure and create a collection
    let config = CollectionConfigBuilder::default()
        .collection_name("person")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .payload_table_schema("create table person (rowid integer primary key, name text)")
        .build()
        .unwrap();

    match vlite.create_collection(config) {
        Ok(_) => {
            // Step 4: Prepare vector points with payloads
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
                    .payload_insert_query("insert into person(name) values ('Charlie')")
                    .build()
                    .unwrap(),

                InsertPoint::builder()
                    .collection_name("person")
                    .id(5)
                    .vector(vec![17.0, 11.0, 9.0, 4.0])
                    .payload_insert_query("insert into person(name) values ('David')")
                    .build()
                    .unwrap(),
            ];

            // Step 5: Insert the data points
            for point in points {
                vlite.insert(point).unwrap();
            }

            // Step 6: Run a vector search
            let search_point = SearchPoint::builder()
                .collection_name("person")
                .vector(vec![7.0, 8.0, 9.0, 2.0])
                .top_k(10)
                .payload_search_query("select * from person")
                .build()
                .unwrap();

            let results = vlite.search(search_point).unwrap();
            println!("🔍 Search results: {:?}", results);
        }
        Err(e) => println!("❌ Error creating collection: {:?}", e),
    }
}
```



## Details Example

```rust




use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use vector_xlite::{VectorXLite, types::*};

pub fn run_complex_example(vlite: &VectorXLite, sqlite_conn_pool: Pool<SqliteConnectionManager>) {
    let create_authors_table = r#"
    create table authors (
            id integer primary key,
            name text not null,
            bio text
        );
        "#;

    let sqlite_conn = sqlite_conn_pool.get().unwrap();

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

---
---
use crate::complex_example::run_complex_example;
use crate::simple_example::run_simple_example;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use vector_xlite::{VectorXLite, customizer::SqliteConnectionCustomizer};

mod complex_example;
mod simple_example;

fn main() {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder()
        .max_size(15)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .unwrap();

    let vlite = VectorXLite::new(pool.clone()).unwrap();

    run_complex_example(&vlite, pool);
}


```

## 🧩 API Reference (Summary)

### `VectorXLite::new(conn: rusqlite::Connection)`
Initializes VectorXLite on the given SQLite connection.

---

### `CollectionConfigBuilder`

| Method                        | Description                                       |
| ----------------------------- | ------------------------------------------------- |
| `.collection_name(&str)`      | Sets the logical collection name                  |
| `.distance(DistanceFunction)` | Sets similarity metric (`Cosine`, `L2`, or `Dot`) |
| `.vector_dimension(usize)`    | Defines vector dimensionality                     |
| `.payload_table_schema(&str)` | SQL to create payload table                       |
| `.build()`                    | Builds final config                               |

---

### `InsertPoint` Builder

| Method                        | Description                |
| ----------------------------- | -------------------------- |
| `.collection_name(&str)`      | Collection to insert into  |
| `.id(i64)`                    | Unique point ID            |
| `.vector(Vec<f32>)`           | Vector embedding           |
| `.payload_insert_query(&str)` | SQL to insert payload data |

---

### `SearchPoint` Builder

| Method                        | Description            |
| ----------------------------- | ---------------------- |
| `.collection_name(&str)`      | Collection to search   |
| `.vector(Vec<f32>)`           | Query vector           |
| `.top_k(usize)`               | Number of top results  |
| `.payload_search_query(&str)` | SQL query for payloads |



## 🛠 Troubleshooting

### Persistent storage
Replace Connection::open_in_memory() with a file-backed connection:

```
let conn = Connection::open("vectors.db")?;
```
