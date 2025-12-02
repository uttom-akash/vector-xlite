//! # Predefined Test Fixtures
//!
//! Ready-to-use collection configurations and test data for common scenarios.
//!
//! ## Usage
//!
//! ```rust
//! use common::fixtures::*;
//!
//! #[test]
//! fn test_with_fixture() {
//!     let ctx = TestContext::memory();
//!     let coll = fixtures::simple_3d(&ctx);
//!     // Collection is ready with test data
//! }
//! ```

use super::*;

// ============================================================================
// Collection Fixtures
// ============================================================================

/// Simple 3D collection without payload (most common test case)
pub fn simple_3d(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("simple_3d"))
        .dimension(3)
        .cosine()
        .create()
}

/// Simple 3D collection with L2 distance
pub fn simple_3d_l2(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("simple_3d_l2"))
        .dimension(3)
        .l2()
        .create()
}

/// Simple 3D collection with inner product
pub fn simple_3d_ip(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("simple_3d_ip"))
        .dimension(3)
        .inner_product()
        .create()
}

/// High-dimensional collection (128D, common for small embeddings)
pub fn high_dim_128(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("high_dim_128"))
        .dimension(128)
        .cosine()
        .create()
}

/// High-dimensional collection (384D, sentence-transformers)
pub fn high_dim_384(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("high_dim_384"))
        .dimension(384)
        .cosine()
        .create()
}

/// High-dimensional collection (768D, BERT-like)
pub fn high_dim_768(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("high_dim_768"))
        .dimension(768)
        .cosine()
        .create()
}

/// Collection with basic text payload
pub fn with_text_payload(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("text_payload"))
        .dimension(3)
        .with_payload("name TEXT, description TEXT")
        .create()
}

/// Collection with numeric payload
pub fn with_numeric_payload(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("numeric_payload"))
        .dimension(3)
        .with_payload("value INTEGER, score REAL")
        .create()
}

/// Collection with JSON payload
pub fn with_json_payload(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("json_payload"))
        .dimension(3)
        .with_payload("metadata JSON")
        .create()
}

/// Collection with all common column types
pub fn with_full_payload(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("full_payload"))
        .dimension(3)
        .with_payload("name TEXT NOT NULL, category TEXT, price REAL, stock INTEGER, metadata JSON, created_at TEXT DEFAULT (datetime('now'))")
        .create()
}

/// Users collection (common business scenario)
pub fn users_collection(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("users"))
        .dimension(128)
        .with_payload("username TEXT NOT NULL UNIQUE, email TEXT, role TEXT DEFAULT 'user'")
        .create()
}

/// Products collection (e-commerce scenario)
pub fn products_collection(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("products"))
        .dimension(384)
        .with_payload("name TEXT NOT NULL, category TEXT, price REAL, in_stock INTEGER DEFAULT 1")
        .create()
}

/// Documents collection (RAG scenario)
pub fn documents_collection(ctx: &TestContext) -> Collection {
    ctx.collection(&unique_name("documents"))
        .dimension(768)
        .with_payload("title TEXT, content TEXT, source TEXT, page_num INTEGER")
        .create()
}

// ============================================================================
// Populated Collection Fixtures (with test data)
// ============================================================================

/// Collection pre-populated with 10 simple vectors
pub fn populated_10(ctx: &TestContext) -> Collection {
    let coll = simple_3d(ctx);
    for i in 1..=10 {
        coll.insert_vector(i, vec![i as f32, 0.0, 0.0]);
    }
    coll
}

/// Collection pre-populated with 100 vectors
pub fn populated_100(ctx: &TestContext) -> Collection {
    let coll = simple_3d(ctx);
    for i in 1..=100 {
        let v = (i as f32) / 100.0;
        coll.insert_vector(i, vec![v, v * 0.5, v * 0.25]);
    }
    coll
}

/// Collection with clustered data (3 clusters)
pub fn clustered_data(ctx: &TestContext) -> Collection {
    let coll = simple_3d(ctx);

    // Cluster 1: around (1, 0, 0)
    for i in 1..=10 {
        let noise = (i as f32) * 0.01;
        coll.insert_vector(i, vec![1.0 + noise, noise, noise]);
    }

    // Cluster 2: around (0, 1, 0)
    for i in 11..=20 {
        let noise = ((i - 10) as f32) * 0.01;
        coll.insert_vector(i, vec![noise, 1.0 + noise, noise]);
    }

    // Cluster 3: around (0, 0, 1)
    for i in 21..=30 {
        let noise = ((i - 20) as f32) * 0.01;
        coll.insert_vector(i, vec![noise, noise, 1.0 + noise]);
    }

    coll
}

/// Collection with users and payload data
pub fn populated_users(ctx: &TestContext) -> Collection {
    let coll = ctx.collection(&unique_name("pop_users"))
        .dimension(3)
        .with_payload("name TEXT NOT NULL, role TEXT")
        .create();

    let users = [
        (1, "Alice", "admin"),
        (2, "Bob", "user"),
        (3, "Carol", "user"),
        (4, "David", "moderator"),
        (5, "Eve", "user"),
    ];

    for (id, name, role) in users {
        coll.insert(id)
            .vector(vec![id as f32, 0.0, 0.0])
            .payload(&format!(
                "INSERT INTO {} (rowid, name, role) VALUES (?1, '{}', '{}')",
                coll.name, name, role
            ))
            .execute_ok();
    }

    coll
}

// ============================================================================
// Test Scenario Builders
// ============================================================================

/// Create a scenario for testing similarity search accuracy
pub struct SimilarityScenario<'a> {
    pub collection: Collection<'a>,
    pub query_vector: Vec<f32>,
    pub expected_nearest_id: u64,
}

/// Build a similarity test scenario
pub fn similarity_scenario(ctx: &TestContext) -> SimilarityScenario {
    let coll = simple_3d(ctx);

    // Insert vectors at known positions
    coll.insert_vector(1, vec![1.0, 0.0, 0.0]);
    coll.insert_vector(2, vec![0.0, 1.0, 0.0]);
    coll.insert_vector(3, vec![0.0, 0.0, 1.0]);
    coll.insert_vector(4, vec![0.5, 0.5, 0.0]);

    SimilarityScenario {
        collection: coll,
        query_vector: vec![0.9, 0.1, 0.0],  // Closest to id=1
        expected_nearest_id: 1,
    }
}

/// Create a scenario for testing payload filtering
pub struct FilterScenario<'a> {
    pub collection: Collection<'a>,
    pub filter_sql: String,
    pub expected_count: usize,
}

/// Build a filter test scenario
pub fn filter_scenario(ctx: &TestContext) -> FilterScenario {
    let coll = ctx.collection(&unique_name("filter_scenario"))
        .dimension(3)
        .with_payload("category TEXT, price REAL")
        .create();

    // Electronics - high price
    for i in 1..=5 {
        coll.insert(i)
            .vector(vec![i as f32, 0.0, 0.0])
            .payload(&format!(
                "INSERT INTO {} (rowid, category, price) VALUES (?1, 'electronics', {})",
                coll.name, 100.0 + (i as f64) * 50.0
            ))
            .execute_ok();
    }

    // Clothing - low price
    for i in 6..=10 {
        coll.insert(i)
            .vector(vec![i as f32, 0.0, 0.0])
            .payload(&format!(
                "INSERT INTO {} (rowid, category, price) VALUES (?1, 'clothing', {})",
                coll.name, 20.0 + (i as f64) * 5.0
            ))
            .execute_ok();
    }

    FilterScenario {
        filter_sql: format!("SELECT * FROM {} WHERE category = 'electronics'", coll.name),
        expected_count: 5,
        collection: coll,
    }
}
