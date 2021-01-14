use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use tokio::net::TcpListener;
use socket2::{Domain, Socket, Type};
use std::net::SocketAddr;
use futures::TryFutureExt;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = hello_world::HelloReply {
            message: format!("Hello {}", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "[::]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    // workaround for std::net::TcPListener using too small backlog (256)
    let incoming = {
            let sock = Socket::new(Domain::ipv6(), Type::stream(), None).unwrap();
            sock.set_reuse_address(true).unwrap();
            sock.set_reuse_port(true).unwrap();
            sock.set_nonblocking(true).unwrap();
            sock.bind(&addr.into()).unwrap();
            sock.listen(8192).unwrap();
            let l = TcpListener::from_std(sock.into_tcp_listener())?;
            async_stream::stream! {
                while let item = l.accept().map_ok(|(st,_)|st).await {
                    yield item;
                }
            }
        };

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}
