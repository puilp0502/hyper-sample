use std::convert::Infallible;
use std::net::{SocketAddr, TcpListener};
use std::os::fd::AsRawFd;
use hyper::Server;
use hyper::service::{make_service_fn, service_fn};
use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::TcpNoDelay;
use hyper_start::hello_world;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let fd = listener.as_raw_fd();
    let _ = setsockopt(fd, TcpNoDelay, &true).unwrap();
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::from_tcp(listener).unwrap().serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

