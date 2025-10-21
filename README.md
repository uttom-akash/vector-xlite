# 🧠 sqlite-vectorx

**VectorXLite** — A lightweight SQLite extension for **vector search** with payload support.

This repository demonstrates how to use the `vector_xlite` crate to:
- Create a collection with embeddings.
- Insert vectors and associated payload data.
- Perform fast vector similarity search (e.g., cosine, dot, L2).

---

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
};
use rusqlite::Connection;

fn main() {
    // Step 1: Open SQLite in memory
    let sqlite_connection = Connection::open_in_memory().unwrap();

    // Step 2: Initialize VectorXLite
    let vs = VectorXLite::new(sqlite_connection).unwrap();

    // Step 3: Configure and create a collection
    let config = CollectionConfigBuilder::default()
        .collection_name("person")
        .distance(DistanceFunction::Cosine)
        .vector_dimension(4)
        .payload_table_schema("create table person (rowid integer primary key, name text)")
        .build()
        .unwrap();

    match vs.create_collection(config) {
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
                vs.insert(point).unwrap();
            }

            // Step 6: Run a vector search
            let search_point = SearchPoint::builder()
                .collection_name("person")
                .vector(vec![7.0, 8.0, 9.0, 2.0])
                .top_k(10)
                .payload_search_query("select * from person")
                .build()
                .unwrap();

            let results = vs.search(search_point).unwrap();
            println!("🔍 Search results: {:?}", results);
        }
        Err(e) => println!("❌ Error creating collection: {:?}", e),
    }
}
```

## 🧱 Step-by-Step Breakdown

### 1. Create the Collection

```rust
let config = CollectionConfigBuilder::default()
    .collection_name("person")
    .distance(DistanceFunction::Cosine)
    .vector_dimension(4)
    .payload_table_schema("create table person (rowid integer primary key, name text)")
    .build()
    .unwrap();

vs.create_collection(config).unwrap();
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

vs.insert(point).unwrap();
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

let results = vs.search(search_point).unwrap();
```

This fetches the top-K most similar vectors from the collection, along with their payloads.

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