use std::net::TcpListener;
use std::sync::Arc;
use threadpool::ThreadPool;

use ::response::Response;
use ::request::RequestStream;
use ::handler::Handler;

/// Server that listen for connections on given address
///
/// The server will listen for connections on the given address,
/// create the request and response objects and pass them to the
/// handler to process the request
///
/// #Examples
///
/// ```
/// use std::env;
/// use http_server::HttpServer;
/// use http_server::handler::{ServerHandler, FileMode};
///
/// let root = env::home_dir().unwrap();
/// let handler = ServerHandler::<FileMode>::new(&root);
/// let server = HttpServer::new("127.0.0.1:9000", 4);
///
/// ```
#[allow(dead_code)]
pub struct HttpServer {
    addr: String,
    listener: TcpListener,
    threadpool: ThreadPool,
}

impl HttpServer {
    /// Creates a new instance of HttpServer
    pub fn new(addr: &str, num_threads: usize) -> HttpServer {
        let listener = TcpListener::bind(addr).ok().expect(format!("Could not bind to address {}", addr).as_ref());

        HttpServer {
            addr: addr.to_string(),
            listener: listener,
            threadpool: ThreadPool::new(num_threads),
        }
    }

    /// Start the server with the given handler
    ///
    /// When started, the server will block and listen for connections,
    /// creating the request and response and passing them to the handler
    /// when a client connects
    pub fn start(&self, handler: Box<Handler + Send + Sync>) {
        let arc = Arc::new(handler);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = arc.clone();

                    self.threadpool.execute(move || {
                        let mut request_stream = RequestStream::from_stream(&stream).unwrap();

                        let mut request = match request_stream.requests().next() {
                            Some(request) => request,
                            None => return,
                        };
                        let mut response = Response::from_stream(&stream).unwrap();
                        handler.handle_request(&mut request, &mut response);
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
