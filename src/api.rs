use hyper::{Body, StatusCode};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::WebSocketStream;

use super::*;
use planc_protocol::ServerMessage;

pub async fn route_request(req: Request, ctx: Arc<ServiceContext>) -> Result<Response> {
    // Parse path '/api/<session_id>'
    let path = req.uri().path();
    assert!(path.starts_with("/api"));
    let mut components = path[1..].split('/').skip(1);
    let session_id = match components.next() {
        Some(session_id) => session_id.to_string(),
        None => {
            return Ok(hyper::Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::empty())?);
        }
    };
    if components.next().is_some() {
        return Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?);
    }

    let response = tungstenite::handshake::server::create_response_with_body(&req, Body::empty)?;
    tokio::spawn(
        hyper::upgrade::on(req)
            .then(|upgraded| async move {
                let websocket = WebSocketStream::from_raw_socket(
                    upgraded?,
                    tungstenite::protocol::Role::Server,
                    None,
                )
                .await;
                let mut connection = Connection::new(websocket);
                match ctx.get_session(&session_id) {
                    Ok(session) => session.join(connection).await,
                    Err(err) => {
                        connection
                            .send(&ServerMessage::Error(format!(
                                "Error joining session: {}",
                                err
                            )))
                            .await
                    }
                }
            })
            .map(|result| {
                result.unwrap_or_else(|err| {
                    log::warn!("route_request: {:?}", err);
                });
            }),
    );
    Ok(response)
}
