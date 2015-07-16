# tiny-http-server
Small http server written in rust

This is an experiment I started to learn the Rust language.

I decided to make an Http server because it is a good challenge and I had a lot of fun building it.

To start the server, just clone this repo and run `$ cargo run` and the it will start listening at port 9000.

Currently, you can only change the host, port, the server mode (list directories or only serve files)
and the server root by editing `main.rs` but command line arguments will eventually be added.

```rust
extern crate http_server;

use std::env;

use http_server::http::HttpServer;

// Import the handler and the two modes, Directory and File
// FileKind will only serve files and return 404 for directories
// while DirectoryKind will list directories
use http_server::http::handler::{ServerHandler, DirectoryKind, FileKind};

fn main() {
    // Get the current path to use as the server root
    let path = env::current_dir().unwrap();
    
    // Here the handler is created to handle the requests
    // Change to FileKind to only serve files
    let handler = ServerHandler::<DirectoryKind>::new(path);
    
    // Set the host and port when creating the server
    let server: HttpServer = HttpServer::new("127.0.0.1:9000", Box::new(handler));
    
    // Start the server
    server.start();
    
    // Go check your browser
}
```
