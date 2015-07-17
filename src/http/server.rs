use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use super::{Request, Response};
use super::handler::Handler;

#[allow(dead_code)]
pub struct HttpServer {
    addr: String,
    listener: TcpListener,
}

impl HttpServer {
    pub fn new(addr: &str) -> HttpServer {
        let listener = TcpListener::bind(addr).ok().expect(format!("Could not bind to address {}", addr).as_ref());

        HttpServer {
            addr: addr.to_string(),
            listener: listener,
        }
    }

    pub fn start(&self, handler: Box<Handler + Send + Sync>) {
        // let handler = Box::new(handler);
        let arc = Arc::new(handler);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = arc.clone();

                    thread::spawn(move || {
                        let mut request = match Request::create(stream.try_clone().ok().expect("Failed to clone request stream")) {
                            Ok(request) => request,
                            Err(error) => panic!(format!("Failed to parse request: {}", error)),
                        };
                        let mut response = Response::new(stream.try_clone().ok().expect("Failed to clone response stream"));
                        handler.handle(&mut request, &mut response);
                    });
                },
                Err(error) => panic!(error),
            }
        }
    }

    pub fn stop(&self) {
        drop(&self.listener);
    }
}
