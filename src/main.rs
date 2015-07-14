extern crate http_server;

use http_server::http::{HttpServer, Response};

fn main() {
    let server: HttpServer = HttpServer::new("127.0.0.1:9999");
    let handler = |req| {
        println!("{:?}", req);
        Response
    };
    server.start(Box::new(handler));
}
