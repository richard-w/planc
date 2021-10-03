use super::*;
use http::StatusCode;
use include_dir::{include_dir, Dir};

pub async fn route_request(req: Request) -> Result<Response> {
    const WEB_DIR: Dir = include_dir!("web/dist/planc");

    let uri = req.uri();
    assert!(uri.path().starts_with('/'));
    let path = &uri.path()[1..];
    let contents = if let Some(file) = WEB_DIR.get_file(path) {
        Some(file.contents)
    } else if let Some(file) = WEB_DIR.get_file("index.html") {
        Some(file.contents)
    } else {
        None
    };

    if let Some(contents) = contents {
        hyper::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(contents))
            .map_err(|err| err.into())
    } else {
        hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("NOT FOUND"))
            .map_err(|err| err.into())
    }
}
