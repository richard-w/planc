extern crate clap;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate include_dir;
extern crate log;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_tungstenite;

mod api;
mod connection;
mod context;
mod error;
mod session;
mod web;

pub use self::connection::*;
pub use self::context::*;
pub use self::error::*;
pub use self::session::*;

use hyper::body::Body;
use hyper::server::conn::AddrStream;
use hyper::server::Server;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
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
        .arg(clap::Arg::with_name("max_sessions")
            .value_name("max_sessions")
            .help("Maximum number of concurrent sessions")
            .long("max-sessions")
            .default_value("8")
        )
        .arg(clap::Arg::with_name("max_users")
            .value_name("max_users")
            .help("Maximum number of users in a session")
            .long("max-users")
            .default_value("16")
        )
        .get_matches();

    let arg_bind_address = args.value_of("bind_address").unwrap();
    let arg_bind_port = args.value_of("bind_port").unwrap();
    let arg_max_sessions = args.value_of("max_sessions").unwrap().parse().expect("Failed to parse max_sessions");
    let arg_max_users = args.value_of("max_users").unwrap().parse().expect("Failed to parse max_users");
    log::info!("Binding to {}:{}", arg_bind_address, arg_bind_port);

    let bind_address: IpAddr = arg_bind_address
        .parse()
        .expect("Failed to parse bind address");
    let bind_port: u16 = arg_bind_port.parse().expect("Failed to parse bind port");
    let socket_address = SocketAddr::new(bind_address, bind_port);

    let ctx = Arc::new(ServiceContext::new(ServiceContextConfig {
        max_sessions: arg_max_sessions,
        max_users: arg_max_users,
    }));
    let server = Server::bind(&socket_address).serve(MakeService::new(ctx));
    log::info!("Server started");

    server.await.expect("Server task failure");
}

struct MakeService {
    ctx: Arc<ServiceContext>,
}

impl MakeService {
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }
}

impl hyper::service::Service<&AddrStream> for MakeService {
    type Response = Service;
    type Error = BoxedError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _conn: &AddrStream) -> Self::Future {
        let ctx = self.ctx.clone();
        Box::pin(async move { Ok(Service::new(ctx)) })
    }
}

struct Service {
    ctx: Arc<ServiceContext>,
}

impl Service {
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }
}

impl hyper::service::Service<Request> for Service {
    type Response = Response;
    type Error = BoxedError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let ctx = self.ctx.clone();
        Box::pin(async move { route_request(req, ctx).await })
    }
}

async fn route_request(req: Request, ctx: Arc<ServiceContext>) -> Result<Response> {
    let path = req.uri().path();
    assert!(path.starts_with('/'));

    match path[1..].split('/').next() {
        Some("api") => api::route_request(req, ctx).await,
        _ => web::route_request(req).await,
    }
}
