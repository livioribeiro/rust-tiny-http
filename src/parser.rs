use std::error::Error;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::net::TcpStream;

use regex::Regex;

use super::headers::Headers;
use super::query::Query;

pub fn parse_request(buf_reader: &mut BufReader<TcpStream>)
        -> Result<Option<(String, String, String, Option<Query>, Headers)>, Box<Error>> {

    let mut request_line = String::new();
    let bytes_read = try!(buf_reader.read_line(&mut request_line));

    if bytes_read == 0 || request_line.is_empty() {
        return Ok(None);
    }

    let malformed_request_error = io::Error::new(ErrorKind::InvalidInput, "Malformed Request");

    let re = Regex::new(
        r"^(?P<method>[A-Z]*?) (?P<path>[^\?]+)(\?(?P<query>.*?))? HTTP/(?P<version>\d\.\d)\r\n$"
    ).unwrap();

    let (method, path, version, query) = match re.captures(&request_line) {
        Some(cap) => {
            let method = cap.name("method").unwrap();
            let path = cap.name("path").unwrap();
            let version = cap.name("version").unwrap();
            let query = cap.name("query").map(|q| Query::from_str(q));
            (method, path, version, query)
        },
        None => return Err(Box::new(malformed_request_error)),
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
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok(Some((version.to_owned(), method.to_owned(), path.to_owned(), query, headers)))
}
