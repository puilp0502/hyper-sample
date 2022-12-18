use std::convert::Infallible;
use futures::stream::StreamExt;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::path::PathBuf;
use futures::Stream;
use hyper::body::Bytes;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

fn sanitize_path(p: PathBuf) -> PathBuf {
    p.iter().filter(|&x| {
        x.to_string_lossy() != ".." && x.to_string_lossy() != "/"
    }).collect()
}


pub async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    if req.method() != Method::GET {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }
    let file_path = PathBuf::from(req.uri().path());
    let sanitized_path = sanitize_path(file_path);
    let f = File::open(&sanitized_path).await;
    if let Ok(f) = f {
        let stream = ReaderStream::new(f);
        let converted: Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send> = Box::new(stream.map(|x| {
            x.map_err(|io_error| Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>)
        }));

        *response.body_mut() = Body::from(converted);
    } else {
        *response.status_mut() = StatusCode::NOT_FOUND;
        *response.body_mut() = Body::from("404 not found".to_string());
    }
    Ok(response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parent_traversal_absolute() {
        let pb = PathBuf::from("/../../../../etc/passwd");
        let pb = sanitize_path(pb);
        let serve_root = PathBuf::from("/home/frank");

        assert_eq!(pb, PathBuf::from("etc/passwd"));
        assert_eq!(serve_root.join(pb), PathBuf::from("/home/frank/etc/passwd"));
    }
    #[test]
    fn test_parent_traversal_relative() {
        let pb = PathBuf::from("../../../../etc/passwd");
        let pb = sanitize_path(pb);
        let serve_root = PathBuf::from("/home/frank");

        assert_eq!(pb, PathBuf::from("etc/passwd"));
        assert_eq!(serve_root.join(pb), PathBuf::from("/home/frank/etc/passwd"));
    }
}