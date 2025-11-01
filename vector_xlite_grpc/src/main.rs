// use tonic::transport::Server;

// use crate::hello::{
//     HelloReply, HelloRequest,
//     greeter_server::{Greeter, GreeterServer},
// };

// pub mod hello {
//     tonic::include_proto!("hello");
// }

// #[derive(Default)]
// pub struct GreeterServiceImpl;

// #[tonic::async_trait]
// impl Greeter for GreeterServiceImpl {
//     async fn say_hello(
//         &self,
//         request: tonic::Request<HelloRequest>,
//     ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
//         let reply = HelloReply {
//             message: format!("Hello {}!", request.into_inner().name),
//         };
//         Ok(tonic::Response::new(reply))
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse()?;
//     let greeter = GreeterServiceImpl::default();

//     Server::builder()
//         .add_service(GreeterServer::new(greeter))
//         .serve(addr)
//         .await?;

//     Ok(())
// }
