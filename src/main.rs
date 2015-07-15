extern crate http_server;

use std::io::Write;

use http_server::http::{HttpServer, Request, Response};

fn main() {
    let server: HttpServer = HttpServer::new("127.0.0.1:9999");
    let handler = |req: Request, mut res: Response| {
        res.start();
        res.write("<html><body><h1>It Works!</h1></body></html>".as_bytes()).unwrap();
        res.flush().unwrap();
    };
    server.start(Box::new(handler));
}
