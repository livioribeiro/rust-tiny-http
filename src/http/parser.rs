use std::io::{self, BufRead, BufReader, ErrorKind};
use std::net::TcpStream;

use regex::Regex;

use super::headers::Headers;
use super::query::Query;

// TODO: Add fragment (#) support
pub fn parse_request(buf_reader: &mut BufReader<TcpStream>)
        -> io::Result<(String, String, String, Query, Headers)> {

    let mut request_line = String::new();

    try!(buf_reader.read_line(&mut request_line));

    let malformed_request_error = io::Error::new(ErrorKind::InvalidInput, "Malformed Request");

    let re = Regex::new(r"^(?P<method>[A-Z]*?) (?P<path>[^\?]+)(\?(?P<query>.*?))? HTTP/(?P<version>\d\.\d)\r\n$").unwrap();
    let (method, path, version, query) = match re.captures(&request_line) {
        Some(cap) => {
            let method = cap.name("method").unwrap();
            let path = cap.name("path").unwrap();
            let version = cap.name("version").unwrap();
            let query = match cap.name("query") {
                Some(q) => Query::from_str(q),
                None => Query::new(),
            };
            (method, path, version, query)
        },
        None => return Err(malformed_request_error),
    };

    let mut headers = Headers::new();

    for line in buf_reader.lines() {
        match line {
            Ok(l) => {
                if l.trim().len() == 0 {
                    break;
                }
                headers.parse(&l);
            },
            Err(e) => return Err(e),
        }
    }

    Ok((version.to_owned(), method.to_owned(), path.to_owned(), query, headers))
}
