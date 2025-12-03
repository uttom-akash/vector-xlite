use crate::conversions::*;
use crate::proto::{self as pb, vector_x_lite_pb_server::VectorXLitePb};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use vector_xlite::VectorXLite;
use vector_xlite::snapshot::{SnapshotChunk, SnapshotConfig, SnapshotExporter, SnapshotImporter};
use vector_xlite::types::{CollectionConfig, InsertPoint, SearchPoint};

pub struct VectorXLiteGrpc {
    vxlite: VectorXLite,
    pool: Pool<SqliteConnectionManager>,
    index_file_paths: Vec<String>,
}

impl VectorXLiteGrpc {
    pub fn new(connection_pool: Pool<SqliteConnectionManager>) -> Self {
        let inner = VectorXLite::new(connection_pool.clone()).expect("failed to setup vector db.");

        VectorXLiteGrpc {
            vxlite: inner,
            pool: connection_pool,
            index_file_paths: Vec::new(),
        }
    }

    /// Creates a new VectorXLiteGrpc with specified index file paths for snapshot restore.
    pub fn with_index_paths(connection_pool: Pool<SqliteConnectionManager>, index_paths: Vec<String>) -> Self {
        let inner = VectorXLite::new(connection_pool.clone()).expect("failed to setup vector db.");

        VectorXLiteGrpc {
            vxlite: inner,
            pool: connection_pool,
            index_file_paths: index_paths,
        }
    }
}

#[tonic::async_trait]
impl VectorXLitePb for VectorXLiteGrpc {
    async fn create_collection(
        &self,
        req: Request<pb::CollectionConfigPb>,
    ) -> Result<Response<pb::EmptyPb>, Status> {
        let cfg = req.into_inner();
        let cfg = CollectionConfig::try_from(cfg).map_err(|e| Status::invalid_argument(e))?;

        self.vxlite
            .create_collection(cfg)
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(pb::EmptyPb {}))
    }

    async fn insert(
        &self,
        req: Request<pb::InsertPointPb>,
    ) -> Result<Response<pb::EmptyPb>, Status> {
        let ip = req.into_inner();
        let point = InsertPoint::try_from(ip).map_err(|e| Status::invalid_argument(e))?;

        self.vxlite
            .insert(point)
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(pb::EmptyPb {}))
    }

    async fn search(
        &self,
        req: Request<pb::SearchPointPb>,
    ) -> Result<Response<pb::SearchResponsePb>, Status> {
        let sp = req.into_inner();
        let search_point = SearchPoint::try_from(sp).map_err(|e| Status::invalid_argument(e))?;

        let results = self
            .vxlite
            .search(search_point)
            .map_err(|e| Status::internal(e.to_string()))?;

        let pb_results = results
            .into_iter()
            .map(|row_map| {
                // expect rowid and distance presence (planner/executor should provide names)
                let rowid = row_map
                    .get("rowid")
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                let distance = row_map
                    .get("distance")
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                build_search_item(rowid, distance, row_map)
            })
            .collect();

        Ok(Response::new(pb::SearchResponsePb {
            results: pb_results,
        }))
    }

    async fn collection_exists(
        &self,
        req: Request<pb::CollectionExistsRequestPb>,
    ) -> Result<Response<pb::CollectionExistsResponsePb>, Status> {
        let collection_name = req.into_inner().collection_name;

        let exists = self
            .vxlite
            .collection_exists(&collection_name)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(pb::CollectionExistsResponsePb { exists }))
    }

    /// Streaming response type for export_snapshot
    type ExportSnapshotStream = ReceiverStream<Result<pb::SnapshotChunkPb, Status>>;

    /// Export a consistent snapshot of the database and index files.
    ///
    /// Streams snapshot chunks for Raft FSM to persist. The first chunk contains
    /// metadata about the snapshot, followed by file data chunks.
    async fn export_snapshot(
        &self,
        req: Request<pb::ExportSnapshotRequestPb>,
    ) -> Result<Response<Self::ExportSnapshotStream>, Status> {
        let request = req.into_inner();
        let config: SnapshotConfig = request.into();

        // Create exporter
        let exporter = SnapshotExporter::new(self.pool.clone(), config);

        // Create a channel for streaming chunks
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // Spawn task to export and send chunks
        let export_result = exporter.export();
        tokio::spawn(async move {
            match export_result {
                Ok(chunk_iter) => {
                    for chunk in chunk_iter {
                        let pb_chunk: pb::SnapshotChunkPb = chunk.into();
                        if tx.send(Ok(pb_chunk)).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Status::internal(e.to_string()))).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// Import a snapshot and restore the database and index files.
    ///
    /// Accepts streamed snapshot chunks and atomically restores the database.
    /// Uses a temp-file-then-replace strategy to ensure data integrity.
    async fn import_snapshot(
        &self,
        req: Request<tonic::Streaming<pb::SnapshotChunkPb>>,
    ) -> Result<Response<pb::ImportSnapshotResponsePb>, Status> {
        let mut stream = req.into_inner();

        // Collect all chunks
        let mut chunks: Vec<SnapshotChunk> = Vec::new();
        while let Some(chunk_result) = stream.message().await? {
            chunks.push(chunk_result.into());
        }

        // Create importer and restore
        let importer = SnapshotImporter::with_defaults(self.pool.clone())
            .with_index_paths(self.index_file_paths.clone());

        let result = importer
            .import(chunks.into_iter())
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(result.into()))
    }
}
