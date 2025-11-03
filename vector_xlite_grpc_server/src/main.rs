use std::env;

use dotenvy::dotenv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Server;
use vector_xlite::customizer::SqliteConnectionCustomizer;
use vector_xlite_grpc::{
    proto::vector_x_lite_pb_server::VectorXLitePbServer, vector_xlite_grpc::VectorXLiteGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder()
        .max_size(15)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .unwrap();

    let addr = env::var("GRPC_ADDR")
        .unwrap_or(String::from("[::1]:50051"))
        .parse()?;

    let vxlite = VectorXLiteGrpc::new(pool);

    Server::builder()
        .add_service(VectorXLitePbServer::new(vxlite))
        .serve(addr)
        .await?;

    Ok(())
}
