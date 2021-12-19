use super::*;
use http::StatusCode;
use include_dir::{include_dir, Dir};

pub async fn route_request(req: Request) -> Result<Response> {
    const WEB_DIR: Dir = include_dir!("web/dist/planc");

    let uri = req.uri();
    assert!(uri.path().starts_with('/'));
    let path = &uri.path()[1..];

    let response = WEB_DIR.get_file(path)
        .map(|file| {
            // Explicit request for an existing file.
            let content_type = match path.rsplit('.').next().unwrap() {
                "html" => "text/html",
                "js" => "text/javascript",
                "css" => "text/css",
                "txt" => "text/plain",
                _ => "application/octet-stream",
            };
            hyper::Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(Body::from(file.contents()))
        })
        .or_else(|| {
            // Fallback path just returns index.html so we can handle most routing in the frontend.
            WEB_DIR.get_file("index.html")
                .map(|file| {
                    hyper::Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Body::from(file.contents()))
                })
        })
        .unwrap_or_else(|| {
            // Fallback for when index.html does not exist, which may happen in some development
            // setups.
            hyper::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("NOT FOUND"))
        })?;

    Ok(response)
}
