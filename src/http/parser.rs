use std::error::Error;
use std::io::{self, BufRead, BufReader};
use std::net::TcpStream;

use super::request::Request;
use super::headers::Headers;

pub fn parse_request(stream: TcpStream) -> Result<Request, Box<Error>> {
    let mut buf_reader = BufReader::new(stream.try_clone().ok().expect("Failed to clone parsing stream"));
    let mut line = String::new();

    buf_reader.read_line(&mut line).ok().expect("Failed to read request line");
    if line.is_empty() {
        return Err(Box::new(io::Error::new(io::ErrorKind::Other, "bad request")));
    }

    let first_line: Vec<_> = line.split(' ').collect();
    let method = first_line[0];

    let version: Vec<_> = first_line[2].trim().split('/').collect();
    let version = version[1];

    let path_query: Vec<_> = first_line[1].split("?").collect();
    let path = path_query[0];
    let query: Option<&str>;
    if path_query.len() > 1 {
        query = Some(path_query[1]);
    }
    else {
        query = None;
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
            Err(error) => panic!("Error reading headers: {}", error),
        }
    }

    Ok(Request::new(version, method, path, query, headers, stream))
}
