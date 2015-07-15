use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use super::request::Request;
use super::response::Response;
use super::parser;

#[allow(dead_code)]
pub struct HttpServer {
    addr: String,
    listener: TcpListener,
}

impl HttpServer {
    pub fn new(addr: &str) -> HttpServer {
        let listener = TcpListener::bind(addr).unwrap();

        HttpServer {
            addr: addr.to_string(),
            listener: listener,
        }
    }

    pub fn start(&self, handler: Box<Fn(Request, Response) + Send + Sync>) {
        let arc = Arc::new(handler);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = arc.clone();
                    thread::spawn(move || {
                        let request = parser::parse_request(stream.try_clone().unwrap());
                        let response = Response::new("HTTP/1.0", 200, "OK", stream.try_clone().unwrap());
                        handler(request, response);
                    });
                },
                Err(_) => {}
            }
        }
    }

    pub fn stop(&self) {
        drop(&self.listener);
    }
}
