use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use conduit::Method;

use super::request::Request;
use super::headers::Headers;


pub fn parse_request(stream: TcpStream) -> Request {
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();

    buf_reader.read_line(&mut line).unwrap();

    let first_line: Vec<_> = line.split(' ').collect();
    let method = Some(first_line[0]);

    let me = match method {
        Some("GET") => Method::Get,
        Some("PUT") => Method::Put,
        Some("POST") => Method::Post,
        Some("DELETE") => Method::Delete,
        Some("HEAD") => Method::Head,
        Some("CONNECT") => Method::Connect,
        Some("OPTIONS") => Method::Options,
        Some("TRACE") => Method::Trace,
        Some("PATCH") => Method::Patch,
        Some("PURGE") => Method::Purge,
        _ => Method::Other("UNKNOWN"),
    };

    let version: Vec<_> = first_line[2].trim().split('/').collect();
    let version = version[1];

    let path_query: Vec<_> = first_line[1].split("?").collect();
    let path = path_query[0];
    let query: &str;
    if path_query.len() > 1 {
        query = path_query[1];
    }
    else {
        query = "";
    }

    let mut headers = Headers::new();

    for line in buf_reader.lines() {
        match line {
            Ok(l) => {
                if l.trim().len() == 0 {
                    break;
                }
                headers.parse(&l);
            },
            _ => {},
        }
    }

    Request::new(version, me, path, Some(query), headers, stream)
}
