extern crate clap;
extern crate env_logger;
extern crate http;
extern crate hyper;
extern crate include_dir;
extern crate log;
extern crate serde;
extern crate tokio;

mod web;

use hyper::body::Body;
use hyper::server::conn::AddrStream;
use hyper::server::Server;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
type Request = hyper::Request<Body>;
type Response = hyper::Response<Body>;
type Result<T> = std::result::Result<T, BoxedError>;

#[tokio::main]
async fn main() {
    // Initialize logging.
    env_logger::init();
    log::info!("Initializing application");

    // Parse command line arguments
    #[rustfmt::skip]
    let args = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .arg(clap::Arg::with_name("bind_address")
            .value_name("bind_address")
            .help("HTTP listener address")
            .short("a")
            .long("bind-address")
            .required(true)
        )
        .arg(clap::Arg::with_name("bind_port")
            .value_name("bind_port")
            .help("HTTP listener port")
            .short("p")
            .long("bind-port")
            .required(true)
        )
        .get_matches();

    let arg_bind_address = args.value_of("bind_address").unwrap();
    let arg_bind_port = args.value_of("bind_port").unwrap();
    log::info!("Binding to {}:{}", arg_bind_address, arg_bind_port);

    let bind_address: IpAddr = arg_bind_address
        .parse()
        .expect("Failed to parse bind address");
    let bind_port: u16 = arg_bind_port.parse().expect("Failed to parse bind port");
    let socket_address = SocketAddr::new(bind_address, bind_port);

    let server = Server::bind(&socket_address).serve(MakeService);
    log::info!("Server started");

    server.await.expect("Server task failure");
}

struct MakeService;

impl hyper::service::Service<&AddrStream> for MakeService {
    type Response = Service;
    type Error = BoxedError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _conn: &AddrStream) -> Self::Future {
        Box::pin(async move { Ok(Service) })
    }
}

struct Service;

impl hyper::service::Service<Request> for Service {
    type Response = Response;
    type Error = BoxedError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        Box::pin(async move { route_request(req).await })
    }
}

async fn route_request(req: Request) -> Result<Response> {
    web::route_request(req).await
}
