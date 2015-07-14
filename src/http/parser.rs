use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use super::request::Request;
use super::headers::Headers;


pub fn parse_request(stream: TcpStream) -> Request {
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();

    buf_reader.read_line(&mut line).unwrap();

    let first_line: Vec<_> = line.split(" ").collect();
    let method = first_line[0].trim();
    let version = first_line[2].trim();

    let path_query: Vec<_> = first_line[1].split("?").collect();
    let path = path_query[0];
    let query = path_query[1];

    let mut headers = Headers::new();

    for line in buf_reader.lines() {
        match line {
            Ok(l) => {
                if l.trim().len() == 0 {
                    break;
                }
                headers.parse(l);
            },
            _ => {},
        }
    }

    Request::new(method, path, query, version, headers, stream)
}
