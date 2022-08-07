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

use anyhow::{Error, Result};
use clap::Parser;
use futures::prelude::*;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

type Request = hyper::Request<hyper::Body>;
type Response = hyper::Response<hyper::Body>;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(version, author, about)]
struct Args {
    /// HTTP listener address
    #[clap(long, short = 'a')]
    bind_address: String,
    /// HTTP listener port
    #[clap(long, short = 'p')]
    bind_port: u16,
    /// Maximum number of concurrent sessions
    #[clap(long, default_value_t = 8)]
    max_sessions: usize,
    /// Maximum number of users in a session
    #[clap(long, default_value_t = 16)]
    max_users: usize,
}

#[tokio::main]
async fn main() {
    // Initialize logging.
    env_logger::init();
    log::info!("main: Initializing application");

    // Parse command line arguments
    let args = Args::parse();

    // Create socket address from command line arguments
    log::info!("main: Binding to {}:{}", args.bind_address, args.bind_port);
    let bind_address: std::net::IpAddr = args
        .bind_address
        .parse()
        .expect("Failed to parse bind address");
    let socket_address = std::net::SocketAddr::new(bind_address, args.bind_port);

    // Create service context config and start server
    let ctx = Arc::new(ServiceContext::new(ServiceContextConfig {
        max_sessions: args.max_sessions,
        max_users: args.max_users,
    }));
    let server = hyper::server::Server::bind(&socket_address).serve(MakeService::new(ctx));
    log::info!("main: Server started");

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

impl hyper::service::Service<&hyper::server::conn::AddrStream> for MakeService {
    type Response = Service;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _conn: &hyper::server::conn::AddrStream) -> Self::Future {
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
    type Error = Error;
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
