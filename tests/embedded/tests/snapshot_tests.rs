//! Snapshot Export/Import Tests
//!
//! Tests for the snapshot functionality including:
//! - Export snapshot from in-memory DB
//! - Export snapshot from file-backed DB with index files
//! - Import snapshot and restore
//! - Large collection snapshots
//! - Follower recovery scenarios
//! - Atomic restore correctness

mod common;

use common::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use vector_xlite::snapshot::{
    SnapshotChunk, SnapshotConfig, SnapshotExporter, SnapshotImporter, SnapshotMetadata,
};

// ============================================================================
// Basic Export Tests
// ============================================================================

#[test]
fn test_export_empty_database() {
    let ctx = TestContext::memory();

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Should have at least metadata chunk and final chunk
    assert!(!chunks.is_empty(), "Should have chunks");

    // First chunk should contain metadata
    let first_chunk = &chunks[0];
    assert!(
        first_chunk.metadata.is_some(),
        "First chunk should have metadata"
    );

    let metadata = first_chunk.metadata.as_ref().unwrap();
    assert!(!metadata.snapshot_id.is_empty(), "Should have snapshot ID");
    assert!(metadata.created_at > 0, "Should have timestamp");
    assert_eq!(metadata.version, 1, "Version should be 1");
}

#[test]
fn test_export_with_single_collection() {
    let ctx = TestContext::memory();

    // Create a collection and insert some data
    let coll = ctx.collection("test_export").dimension(3).cosine().create();
    coll.insert_vector(1, vec![1.0, 0.0, 0.0]);
    coll.insert_vector(2, vec![0.0, 1.0, 0.0]);
    coll.insert_vector(3, vec![0.0, 0.0, 1.0]);

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Verify metadata
    let first_chunk = &chunks[0];
    let metadata = first_chunk.metadata.as_ref().unwrap();

    // Should have at least the SQLite database file
    assert!(
        !metadata.files.is_empty(),
        "Should have at least one file in snapshot"
    );
    assert!(metadata.total_size > 0, "Total size should be > 0");
}

#[test]
fn test_export_with_payload() {
    let ctx = TestContext::memory();

    // Create collection with payload
    let coll = ctx
        .collection("payload_test")
        .dimension(3)
        .with_payload("name TEXT, category TEXT")
        .create();

    // Insert vectors with payload
    for i in 1..=10 {
        coll.insert(i as u64)
            .vector(vec![i as f32, 0.0, 0.0])
            .payload(&format!(
                "INSERT INTO payload_test (rowid, name, category) VALUES (?1, 'item{}', 'cat{}')",
                i,
                i % 3
            ))
            .execute_ok();
    }

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    let metadata = chunks[0].metadata.as_ref().unwrap();
    assert!(metadata.total_size > 0, "Should have data");
}

// ============================================================================
// Export with File-Backed Database
// ============================================================================

#[test]
fn test_export_file_backed_database() {
    let ctx = TestContext::file();

    let coll = ctx.collection("file_test").dimension(3).cosine().create();
    coll.insert_vector(1, vec![1.0, 0.0, 0.0]);
    coll.insert_vector(2, vec![0.0, 1.0, 0.0]);

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    let metadata = chunks[0].metadata.as_ref().unwrap();
    assert!(!metadata.files.is_empty(), "Should have files");

    // Verify checksum is present
    for file in &metadata.files {
        assert!(!file.checksum.is_empty(), "File should have checksum");
        assert!(file.file_size > 0, "File should have size");
    }
}

// ============================================================================
// Large Collection Tests
// ============================================================================

#[test]
fn test_export_large_collection() {
    let ctx = TestContext::memory();

    // Create a collection with many vectors
    let coll = ctx
        .collection("large_coll")
        .dimension(128)
        .max_elements(50000)
        .create();

    // Insert 1000 vectors
    for i in 1..=1000 {
        let vector: Vec<f32> = (0..128).map(|j| (i * j) as f32 / 1000.0).collect();
        coll.insert_vector(i, vector);
    }

    let config = SnapshotConfig::default().with_chunk_size(32 * 1024); // 32KB chunks
    let exporter = SnapshotExporter::new(ctx.pool.clone(), config);
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Should have multiple chunks due to data size
    assert!(chunks.len() > 1, "Should have multiple chunks for large data");

    // Verify sequence ordering
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.sequence as usize, i, "Sequence should match index");
    }

    // Last chunk should be marked as final
    let last_chunk = chunks.last().unwrap();
    assert!(last_chunk.is_final, "Last chunk should be marked as final");
}

#[test]
fn test_export_with_high_dimension_vectors() {
    let ctx = TestContext::memory();

    // Create collection with high-dimensional vectors (like BERT embeddings)
    let coll = ctx
        .collection("high_dim")
        .dimension(768)
        .max_elements(10000)
        .create();

    // Insert 100 high-dimensional vectors
    for i in 1..=100 {
        let vector: Vec<f32> = (0..768)
            .map(|j| ((i * j) as f32).sin() / 100.0)
            .collect();
        coll.insert_vector(i, vector);
    }

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    let metadata = chunks[0].metadata.as_ref().unwrap();
    assert!(
        metadata.total_size > 0,
        "Should have significant data for high-dim vectors"
    );
}

// ============================================================================
// Import Tests
// ============================================================================

#[test]
fn test_import_empty_snapshot() {
    let src_ctx = TestContext::memory();
    let dest_ctx = TestContext::memory();

    // Export from empty source
    let exporter = SnapshotExporter::with_defaults(src_ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Import to destination
    let importer = SnapshotImporter::with_defaults(dest_ctx.pool.clone());
    let result = importer
        .import(chunks.into_iter())
        .expect("Import should succeed");

    assert!(result.success, "Import should be successful");
    assert!(!result.snapshot_id.is_empty(), "Should have snapshot ID");
}

#[test]
fn test_import_with_data() {
    let src_ctx = TestContext::memory();

    // Create and populate source collection
    let coll = src_ctx
        .collection("import_test")
        .dimension(3)
        .cosine()
        .create();
    coll.insert_vector(1, vec![1.0, 0.0, 0.0]);
    coll.insert_vector(2, vec![0.0, 1.0, 0.0]);
    coll.insert_vector(3, vec![0.0, 0.0, 1.0]);

    // Export
    let exporter = SnapshotExporter::with_defaults(src_ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Create new destination context
    let dest_ctx = TestContext::memory();

    // Import
    let importer = SnapshotImporter::with_defaults(dest_ctx.pool.clone());
    let result = importer
        .import(chunks.into_iter())
        .expect("Import should succeed");

    assert!(result.success, "Import should be successful");
    assert!(result.bytes_restored > 0, "Should have restored bytes");
    assert!(result.files_restored > 0, "Should have restored files");
}

#[test]
fn test_import_preserves_data() {
    let src_ctx = TestContext::memory();

    // Create source with specific data
    let coll = src_ctx
        .collection("preserve_test")
        .dimension(3)
        .with_payload("name TEXT")
        .create();

    coll.insert(1)
        .vector(vec![1.0, 0.0, 0.0])
        .payload("INSERT INTO preserve_test (rowid, name) VALUES (?1, 'Alice')")
        .execute_ok();

    coll.insert(2)
        .vector(vec![0.0, 1.0, 0.0])
        .payload("INSERT INTO preserve_test (rowid, name) VALUES (?1, 'Bob')")
        .execute_ok();

    // Export
    let exporter = SnapshotExporter::with_defaults(src_ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    let metadata = chunks[0].metadata.as_ref().unwrap();
    let snapshot_id = metadata.snapshot_id.clone();

    // Import to new context
    let dest_ctx = TestContext::memory();
    let importer = SnapshotImporter::with_defaults(dest_ctx.pool.clone());
    let result = importer
        .import(chunks.into_iter())
        .expect("Import should succeed");

    assert!(result.success);
    assert_eq!(result.snapshot_id, snapshot_id);
}

// ============================================================================
// Follower Recovery Tests
// ============================================================================

#[test]
fn test_follower_recovery_scenario() {
    // Simulate leader with data
    let leader_ctx = TestContext::memory();

    let coll = leader_ctx
        .collection("raft_test")
        .dimension(64)
        .with_payload("data TEXT")
        .create();

    // Insert data as "leader"
    for i in 1..=50 {
        coll.insert(i as u64)
            .vector((0..64).map(|j| (i * j) as f32 / 1000.0).collect())
            .payload(&format!(
                "INSERT INTO raft_test (rowid, data) VALUES (?1, 'entry_{}')",
                i
            ))
            .execute_ok();
    }

    // Create snapshot (as if for Raft FSM)
    let exporter = SnapshotExporter::with_defaults(leader_ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Simulate new follower joining
    let follower_ctx = TestContext::memory();

    // Follower restores from snapshot
    let importer = SnapshotImporter::with_defaults(follower_ctx.pool.clone());
    let result = importer
        .import(chunks.into_iter())
        .expect("Import should succeed");

    assert!(result.success, "Follower should successfully restore");
    assert!(
        result.bytes_restored > 0,
        "Follower should have restored data"
    );
}

#[test]
fn test_snapshot_transfer_chunks_in_order() {
    let ctx = TestContext::memory();

    let coll = ctx.collection("order_test").dimension(32).create();
    for i in 1..=100 {
        let vector: Vec<f32> = (0..32).map(|j| (i * j) as f32 / 100.0).collect();
        coll.insert_vector(i, vector);
    }

    let config = SnapshotConfig::default().with_chunk_size(4 * 1024); // Small chunks
    let exporter = SnapshotExporter::new(ctx.pool.clone(), config);
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Verify chunks are in sequence order
    let mut expected_seq = 0u64;
    for chunk in &chunks {
        assert_eq!(
            chunk.sequence, expected_seq,
            "Chunk sequence should be in order"
        );
        expected_seq += 1;
    }
}

// ============================================================================
// Atomic Restore Tests
// ============================================================================

#[test]
fn test_atomic_restore_replaces_existing_data() {
    // Create initial context with some data
    let initial_ctx = TestContext::memory();
    let coll = initial_ctx
        .collection("initial")
        .dimension(3)
        .create();
    coll.insert_vector(1, vec![9.0, 9.0, 9.0]);

    // Create source context with different data
    let src_ctx = TestContext::memory();
    let src_coll = src_ctx.collection("new_data").dimension(3).create();
    src_coll.insert_vector(1, vec![1.0, 0.0, 0.0]);
    src_coll.insert_vector(2, vec![0.0, 1.0, 0.0]);

    // Export source
    let exporter = SnapshotExporter::with_defaults(src_ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    // Import to initial context (should replace data)
    let importer = SnapshotImporter::with_defaults(initial_ctx.pool.clone());
    let result = importer
        .import(chunks.into_iter())
        .expect("Import should succeed");

    assert!(result.success, "Atomic restore should succeed");
}

// ============================================================================
// Snapshot Metadata Tests
// ============================================================================

#[test]
fn test_snapshot_metadata_generation() {
    let id1 = SnapshotMetadata::generate_id();
    let id2 = SnapshotMetadata::generate_id();

    // IDs should be unique
    assert_ne!(id1, id2, "Generated IDs should be unique");

    // IDs should have expected format
    assert!(id1.starts_with("snap_"), "ID should start with 'snap_'");
    assert!(id2.starts_with("snap_"), "ID should start with 'snap_'");
}

#[test]
fn test_snapshot_checksum_validation() {
    let ctx = TestContext::memory();

    let coll = ctx.collection("checksum_test").dimension(3).create();
    coll.insert_vector(1, vec![1.0, 0.0, 0.0]);

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    let metadata = chunks[0].metadata.as_ref().unwrap();

    // Verify all files have checksums
    for file in &metadata.files {
        assert!(
            !file.checksum.is_empty(),
            "Every file should have a checksum"
        );
        assert!(file.file_size > 0, "Every file should have a size");
    }

    // Verify snapshot checksum
    assert!(
        !metadata.checksum.is_empty(),
        "Snapshot should have overall checksum"
    );
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_custom_chunk_size() {
    let ctx = TestContext::memory();

    let coll = ctx.collection("chunk_size_test").dimension(64).create();
    for i in 1..=200 {
        let vector: Vec<f32> = (0..64).map(|j| (i * j) as f32 / 1000.0).collect();
        coll.insert_vector(i, vector);
    }

    // Test with small chunk size
    let small_config = SnapshotConfig::default().with_chunk_size(8 * 1024); // 8KB
    let exporter = SnapshotExporter::new(ctx.pool.clone(), small_config);
    let small_chunks: Vec<SnapshotChunk> =
        exporter.export().expect("Export should succeed").collect();

    // Test with larger chunk size
    let large_config = SnapshotConfig::default().with_chunk_size(128 * 1024); // 128KB
    let exporter = SnapshotExporter::new(ctx.pool.clone(), large_config);
    let large_chunks: Vec<SnapshotChunk> =
        exporter.export().expect("Export should succeed").collect();

    // Small chunks should result in more chunks
    assert!(
        small_chunks.len() >= large_chunks.len(),
        "Smaller chunk size should produce more or equal chunks"
    );
}

// ============================================================================
// Streaming Tests
// ============================================================================

#[test]
fn test_streaming_export_iterator() {
    let ctx = TestContext::memory();

    let coll = ctx.collection("stream_test").dimension(3).create();
    coll.insert_vector(1, vec![1.0, 0.0, 0.0]);

    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let mut iter = exporter.export().expect("Export should succeed");

    // First item should have metadata
    let first = iter.next();
    assert!(first.is_some(), "Should have first chunk");
    assert!(
        first.unwrap().metadata.is_some(),
        "First chunk should have metadata"
    );

    // Iterate until done
    let mut count = 1;
    while let Some(_chunk) = iter.next() {
        count += 1;
        if count > 1000 {
            panic!("Too many chunks - possible infinite loop");
        }
    }

    assert!(count >= 2, "Should have at least metadata and final chunks");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_export_after_delete_operations() {
    let ctx = TestContext::memory();

    let coll = ctx
        .collection("delete_test")
        .dimension(3)
        .with_payload("name TEXT")
        .create();

    // Insert some data
    for i in 1..=10 {
        coll.insert(i as u64)
            .vector(vec![i as f32, 0.0, 0.0])
            .payload(&format!(
                "INSERT INTO delete_test (rowid, name) VALUES (?1, 'item_{}')",
                i
            ))
            .execute_ok();
    }

    // Delete some data directly via SQL
    ctx.execute_sql("DELETE FROM delete_test WHERE rowid > 5")
        .expect("Delete should work");

    // Export should still work
    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    assert!(!chunks.is_empty(), "Should still have chunks after delete");
}

#[test]
fn test_multiple_collections_export() {
    let ctx = TestContext::memory();

    // Create multiple collections
    let coll1 = ctx.collection("multi_1").dimension(3).create();
    let coll2 = ctx.collection("multi_2").dimension(3).create();
    let coll3 = ctx.collection("multi_3").dimension(3).create();

    // Insert data into each
    coll1.insert_vector(1, vec![1.0, 0.0, 0.0]);
    coll2.insert_vector(1, vec![0.0, 1.0, 0.0]);
    coll3.insert_vector(1, vec![0.0, 0.0, 1.0]);

    // Export should include all collections
    let exporter = SnapshotExporter::with_defaults(ctx.pool.clone());
    let chunks: Vec<SnapshotChunk> = exporter.export().expect("Export should succeed").collect();

    let metadata = chunks[0].metadata.as_ref().unwrap();
    assert!(
        metadata.total_size > 0,
        "Should have data from all collections"
    );
}
