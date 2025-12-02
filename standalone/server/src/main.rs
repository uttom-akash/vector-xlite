use clap::Parser;
use dotenvy::dotenv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::env;
use tonic::transport::Server;
use vector_xlite::customizer::SqliteConnectionCustomizer;
use vector_xlite_grpc::{
    proto::vector_x_lite_pb_server::VectorXLitePbServer, vector_xlite_grpc::VectorXLiteGrpc,
};

#[derive(Parser, Debug)]
#[command(name = "vector_xlite_grpc")]
#[command(about = "VectorXLite gRPC Server", long_about = None)]
struct Args {
    /// Port to listen on (overrides GRPC_ADDR env variable)
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args = Args::parse();

    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder()
        .max_size(15)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .unwrap();

    // Priority: command-line arg > env variable > default
    let addr = if let Some(port) = args.port {
        format!("0.0.0.0:{}", port)
    } else {
        env::var("GRPC_ADDR").unwrap_or(String::from("0.0.0.0:50051"))
    };

    let addr = addr.parse()?;

    println!("VectorXLite gRPC server starting on {}", addr);

    let vxlite = VectorXLiteGrpc::new(pool);

    Server::builder()
        .add_service(VectorXLitePbServer::new(vxlite))
        .serve(addr)
        .await?;

    Ok(())
}
