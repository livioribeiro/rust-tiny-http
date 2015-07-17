use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use ::{Request, Response};
use ::handler::Handler;

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
        let arc = Arc::new(handler);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = arc.clone();

                    thread::spawn(move || {
                        let mut request = Request::from_stream(stream.try_clone().unwrap()).unwrap();
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

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.stop();
    }
}
