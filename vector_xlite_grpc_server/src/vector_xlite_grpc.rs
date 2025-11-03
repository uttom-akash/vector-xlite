use crate::conversions::*;
use crate::proto::{self as pb, vector_x_lite_pb_server::VectorXLitePb};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::{Request, Response, Status};
use vector_xlite::VectorXLite;
use vector_xlite::types::{CollectionConfig, InsertPoint, SearchPoint};

pub struct VectorXLiteGrpc {
    vxlite: VectorXLite,
}

impl VectorXLiteGrpc {
    pub fn new(connection_pool: Pool<SqliteConnectionManager>) -> Self {
        let inner = VectorXLite::new(connection_pool).expect("failed to setup vector db.");

        VectorXLiteGrpc { vxlite: inner }
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
}
