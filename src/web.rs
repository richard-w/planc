use super::*;
use http::StatusCode;
use include_dir::{include_dir, Dir};

const WEB_DIR: Dir = include_dir!("web/dist/planc");

pub async fn route_request(req: Request) -> Result<Response> {
    let uri = req.uri();
    assert!(uri.path().starts_with('/'));
    let path = if uri.path() == "/" {
        "index.html"
    } else {
        &uri.path()[1..]
    };

    if let Some(file) = WEB_DIR.get_file(path) {
        hyper::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(file.contents))
            .map_err(|err| err.into())
    } else {
        hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("NOT FOUND"))
            .map_err(|err| err.into())
    }
}
