use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use super::request::{Request, RequestBuilder};
use super::Response;

fn parse_request(stream: &TcpStream) -> Request {
    let mut builder = RequestBuilder::new();

    let mut buf_reader = BufReader::new(stream);
    let mut line = String::new();

    buf_reader.read_line(&mut line).unwrap();

    let first_line: Vec<_> = line.split(" ").collect();
    let method = first_line[0].trim();
    let version = first_line[2].trim();

    let path_query: Vec<_> = first_line[1].split("?").collect();
    let path = path_query[0];

    builder.with_method(method)
        .with_path(path)
        .with_http_version(version);

    if path_query.len() > 1 {
        for q in path_query[1].split("&") {
            let key_value: Vec<_> = q.split("=").collect();
            builder.with_query(key_value[0], key_value[1]);
        }
    }

    for line in buf_reader.lines() {
        match line {
            Ok(l) => {
                if l.trim().len() == 0 {
                    break;
                }
                let header: Vec<_> = l.split(": ").collect();
                for val in header[1].split(",") {
                    builder.with_header(header[0].trim(), val.trim());
                }
            },
            _ => {},
        }
    }

    builder.build()
}

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

    pub fn start(&self, handler: Box<Fn(Request) -> Response + Send + Sync>) {
        let arc = Arc::new(handler);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = arc.clone();
                    thread::spawn(move || {
                        let request = parse_request(&stream);
                        handler(request);
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
