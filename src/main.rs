mod api;
mod connection;
mod context;
mod error;
mod protocol;
mod session;
mod web;

pub use self::connection::*;
pub use self::context::*;
pub use self::error::*;
pub use self::protocol::*;
pub use self::session::*;

use anyhow::{Error, Result};
use clap::Parser;
use futures::prelude::*;
use http_body_util::Full;
use hyper::body::Bytes;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::prelude::*;

type Request = hyper::Request<hyper::body::Incoming>;
type Response = hyper::Response<Full<Bytes>>;

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
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    let tracing_format = ::tracing_subscriber::fmt::layer()
        .json()
        .flatten_event(true)
        .with_span_list(false);
    let tracing_filter = ::tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| ::tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();
    ::tracing_subscriber::registry()
        .with(tracing_format)
        .with(tracing_filter)
        .init();
    ::tracing::info!("init_application");

    // Parse command line arguments
    let args = Args::parse();

    // Create socket address from command line arguments
    ::tracing::info!(
        bind_address = args.bind_address,
        bind_port = args.bind_port,
        "binding_listener"
    );
    let bind_address: std::net::IpAddr = args
        .bind_address
        .parse()
        .expect("Failed to parse bind address");
    let socket_address = std::net::SocketAddr::new(bind_address, args.bind_port);

    // Create service context config
    let ctx = Arc::new(ServiceContext::new(ServiceContextConfig {
        max_sessions: args.max_sessions,
        max_users: args.max_users,
    }));

    // Create tcp listener.
    let tcp_listener = TcpListener::bind(&socket_address).await?;

    loop {
        match tcp_listener.accept().await {
            Ok((tcp_stream, peer_addr)) => {
                ::tracing::info!(peer_addr = peer_addr.to_string(), "incoming_connection");
                let service = Service::new(Arc::clone(&ctx));
                tokio::spawn(async move {
                    let tcp_stream = hyper_util::rt::TokioIo::new(tcp_stream);
                    let result = hyper::server::conn::http1::Builder::new()
                        .serve_connection(tcp_stream, service)
                        .with_upgrades()
                        .await;
                    if let Err(err) = result {
                        ::tracing::warn!(?err, "connection");
                    }
                });
            }
            Err(err) => ::tracing::warn!(?err, "accept"),
        }
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

    fn call(&self, req: Request) -> Self::Future {
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
