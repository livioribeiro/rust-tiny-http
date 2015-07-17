extern crate http_server;
extern crate conduit_mime_types;

use std::env;

use http_server::http::HttpServer;
use http_server::http::handler::{ServerHandler, DirectoryKind, FileKind};

fn main() {
    let path = env::home_dir().unwrap();

    let handler = ServerHandler::<DirectoryKind>::new(path);
    let server: HttpServer = HttpServer::new("127.0.0.1:9000");
    server.start(Box::new(handler));
}
