use crate::proto::{CollectionConfigPb, InsertPointPb, SearchPointPb};
use std::convert::TryFrom;
use vector_xlite::types::{
    CollectionConfig, CollectionConfigBuilder, DistanceFunction, InsertPoint, SearchPoint,
};

impl TryFrom<CollectionConfigPb> for CollectionConfig {
    type Error = String;
    fn try_from(pb: CollectionConfigPb) -> Result<Self, Self::Error> {
        let distance = match pb.distance.to_lowercase().as_str() {
            "cosine" => DistanceFunction::Cosine,
            "l2" => DistanceFunction::L2,
            "ip" => DistanceFunction::IP,
            other => return Err(format!("unknown distance: {}", other)),
        };

        CollectionConfigBuilder::default()
            .collection_name(&pb.collection_name)
            .distance(distance)
            .vector_dimension(pb.vector_dimension as u16)
            .payload_table_schema(&pb.payload_table_schema)
            .index_file_path(pb.index_file_path)
            .build()
            .map_err(|e| e.to_string())
    }
}

impl TryFrom<InsertPointPb> for InsertPoint {
    type Error = String;
    fn try_from(pb: InsertPointPb) -> Result<Self, Self::Error> {
        let mut b = InsertPoint::builder()
            .collection_name(&pb.collection_name)
            .id(pb.id as u64)
            .vector(pb.vector);

        if !pb.payload_insert_query.is_empty() {
            b = b.payload_insert_query(&pb.payload_insert_query);
        }
        b.build().map_err(|e| e.to_string())
    }
}

impl TryFrom<SearchPointPb> for SearchPoint {
    type Error = String;
    fn try_from(pb: SearchPointPb) -> Result<Self, Self::Error> {
        let mut b: vector_xlite::types::SearchPointBuilder = SearchPoint::builder();
        b = b.collection_name(&pb.collection_name);
        b = b.vector(pb.vector);
        b = b.top_k(pb.top_k as i64);
        if !pb.payload_search_query.is_empty() {
            b = b.payload_search_query(&pb.payload_search_query);
        }
        b.build().map_err(|e| e.to_string())
    }
}

// Conversion helpers for responses
use crate::proto::{KeyValuePb, SearchResultItemPb};
use std::collections::HashMap;

pub fn map_payload_to_kvs(map: &HashMap<String, String>) -> Vec<KeyValuePb> {
    map.iter()
        .map(|(k, v)| KeyValuePb {
            key: k.clone(),
            value: v.clone(),
        })
        .collect()
}

pub fn build_search_item(
    rowid: i64,
    distance: f32,
    payload: HashMap<String, String>,
) -> SearchResultItemPb {
    SearchResultItemPb {
        rowid,
        distance,
        payload: map_payload_to_kvs(&payload),
    }
}

// ============================================================================
// Snapshot Conversions
// ============================================================================

use crate::proto::{
    ExportSnapshotRequestPb, ImportSnapshotResponsePb, SnapshotChunkPb, SnapshotFilePb,
    SnapshotFileInfoPb, SnapshotFileTypePb, SnapshotMetadataPb,
};
use vector_xlite::snapshot::{
    FileChunk, ImportResult, SnapshotChunk, SnapshotConfig, SnapshotFileInfo,
    SnapshotFileType, SnapshotMetadata,
};

/// Convert ExportSnapshotRequestPb to SnapshotConfig
impl From<ExportSnapshotRequestPb> for SnapshotConfig {
    fn from(pb: ExportSnapshotRequestPb) -> Self {
        let mut config = SnapshotConfig::default();
        if pb.chunk_size > 0 {
            config = config.with_chunk_size(pb.chunk_size as usize);
        }
        // Note: include_index_files defaults to true, so we only change if explicitly false
        // In proto3, bool defaults to false, so we treat 0/false as "use default"
        config
    }
}

/// Convert SnapshotFileType to SnapshotFileTypePb
impl From<SnapshotFileType> for SnapshotFileTypePb {
    fn from(ft: SnapshotFileType) -> Self {
        match ft {
            SnapshotFileType::SqliteDb => SnapshotFileTypePb::SnapshotFileTypeSqliteDb,
            SnapshotFileType::HnswIndex => SnapshotFileTypePb::SnapshotFileTypeHnswIndex,
            SnapshotFileType::Wal => SnapshotFileTypePb::SnapshotFileTypeWal,
        }
    }
}

/// Convert SnapshotFileTypePb to SnapshotFileType
impl From<SnapshotFileTypePb> for SnapshotFileType {
    fn from(pb: SnapshotFileTypePb) -> Self {
        match pb {
            SnapshotFileTypePb::SnapshotFileTypeSqliteDb => SnapshotFileType::SqliteDb,
            SnapshotFileTypePb::SnapshotFileTypeHnswIndex => SnapshotFileType::HnswIndex,
            SnapshotFileTypePb::SnapshotFileTypeWal => SnapshotFileType::Wal,
            SnapshotFileTypePb::SnapshotFileTypeUnknown => SnapshotFileType::SqliteDb, // Default
        }
    }
}

/// Convert SnapshotFileInfo to SnapshotFileInfoPb
impl From<SnapshotFileInfo> for SnapshotFileInfoPb {
    fn from(info: SnapshotFileInfo) -> Self {
        SnapshotFileInfoPb {
            file_name: info.file_name,
            file_type: SnapshotFileTypePb::from(info.file_type) as i32,
            file_size: info.file_size,
            checksum: info.checksum,
        }
    }
}

/// Convert SnapshotFileInfoPb to SnapshotFileInfo
impl From<SnapshotFileInfoPb> for SnapshotFileInfo {
    fn from(pb: SnapshotFileInfoPb) -> Self {
        SnapshotFileInfo {
            file_name: pb.file_name,
            file_type: SnapshotFileTypePb::try_from(pb.file_type)
                .unwrap_or(SnapshotFileTypePb::SnapshotFileTypeUnknown)
                .into(),
            file_size: pb.file_size,
            checksum: pb.checksum,
        }
    }
}

/// Convert SnapshotMetadata to SnapshotMetadataPb
impl From<SnapshotMetadata> for SnapshotMetadataPb {
    fn from(meta: SnapshotMetadata) -> Self {
        SnapshotMetadataPb {
            snapshot_id: meta.snapshot_id,
            created_at: meta.created_at,
            total_size: meta.total_size,
            files: meta.files.into_iter().map(|f| f.into()).collect(),
            version: meta.version,
            checksum: meta.checksum,
        }
    }
}

/// Convert SnapshotMetadataPb to SnapshotMetadata
impl From<SnapshotMetadataPb> for SnapshotMetadata {
    fn from(pb: SnapshotMetadataPb) -> Self {
        SnapshotMetadata {
            snapshot_id: pb.snapshot_id,
            created_at: pb.created_at,
            total_size: pb.total_size,
            files: pb.files.into_iter().map(|f| f.into()).collect(),
            version: pb.version,
            checksum: pb.checksum,
        }
    }
}

/// Convert FileChunk to SnapshotFilePb
impl From<FileChunk> for SnapshotFilePb {
    fn from(chunk: FileChunk) -> Self {
        SnapshotFilePb {
            file_name: chunk.file_name,
            offset: chunk.offset,
            data: chunk.data,
            is_last_chunk: chunk.is_last_chunk,
        }
    }
}

/// Convert SnapshotFilePb to FileChunk
impl From<SnapshotFilePb> for FileChunk {
    fn from(pb: SnapshotFilePb) -> Self {
        FileChunk {
            file_name: pb.file_name,
            offset: pb.offset,
            data: pb.data,
            is_last_chunk: pb.is_last_chunk,
        }
    }
}

/// Convert SnapshotChunk to SnapshotChunkPb
impl From<SnapshotChunk> for SnapshotChunkPb {
    fn from(chunk: SnapshotChunk) -> Self {
        SnapshotChunkPb {
            metadata: chunk.metadata.map(|m| m.into()),
            file_chunk: chunk.file_chunk.map(|f| f.into()),
            sequence: chunk.sequence,
            is_final: chunk.is_final,
        }
    }
}

/// Convert SnapshotChunkPb to SnapshotChunk
impl From<SnapshotChunkPb> for SnapshotChunk {
    fn from(pb: SnapshotChunkPb) -> Self {
        SnapshotChunk {
            metadata: pb.metadata.map(|m| m.into()),
            file_chunk: pb.file_chunk.map(|f| f.into()),
            sequence: pb.sequence,
            is_final: pb.is_final,
        }
    }
}

/// Convert ImportResult to ImportSnapshotResponsePb
impl From<ImportResult> for ImportSnapshotResponsePb {
    fn from(result: ImportResult) -> Self {
        ImportSnapshotResponsePb {
            success: result.success,
            error_message: result.error_message.unwrap_or_default(),
            snapshot_id: result.snapshot_id,
            bytes_restored: result.bytes_restored,
            files_restored: result.files_restored,
        }
    }
}
