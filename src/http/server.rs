use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use ::{Request, Response};
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
/// let server = HttpServer::new("127.0.0.1:9000");
///
/// ```
#[allow(dead_code)]
pub struct HttpServer {
    addr: String,
    listener: TcpListener,
}

impl HttpServer {
    /// Creates a new instance of HttpServer
    pub fn new(addr: &str) -> HttpServer {
        let listener = TcpListener::bind(addr).ok().expect(format!("Could not bind to address {}", addr).as_ref());

        HttpServer {
            addr: addr.to_string(),
            listener: listener,
        }
    }
    
    /// Start the server with the given handler
    ///
    /// When started, the server will block and listen for connections,
    /// creating the request and response and passing them to the handler
    /// when a client connects
    ///
    /// #Examples
    ///
    /// ```
    /// # use std::env;
    /// # use std::sync::Arc;
    /// # use std::thread; 
    /// use http_server::HttpServer;
    /// # use http_server::handler::{ServerHandler, FileMode};
    /// 
    /// # let root = env::home_dir().unwrap();
    /// let server = HttpServer::new("127.0.0.1:9000"); 
    /// # let arc = Arc::new(server);
    /// # let server = arc.clone();
    /// # thread::spawn(move || {
    /// # let handler = ServerHandler::<FileMode>::new(&root);
    /// server.start(Box::new(handler));
    /// # });
    /// # arc.clone().stop();
    /// ```
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
