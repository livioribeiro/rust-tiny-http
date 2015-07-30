use std::error::Error;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::net::TcpStream;
use regex::Regex;
use url;
use url::format::PathFormatter;
use url::percent_encoding;

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
        r"^(?P<method>[A-Z]*?) (?P<path>.*?) HTTP/(?P<version>\d\.\d)\r\n$"
    ).unwrap();

    let (method, path, version, query) = match re.captures(&request_line) {
        Some(cap) => {
            let method = cap.name("method").unwrap();
            let (path, query, _) = url::parse_path(cap.name("path").unwrap()).unwrap();
            let version = cap.name("version").unwrap();
            let query = query.map(|q| Query::from_str(&q));
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

    // let path = path.iter().fold("".to_string(), |a, b| {
    //     format!("{}/{}", a, b)
    // });

    let path = format!("{}", PathFormatter { path: &path });
    let path = String::from_utf8(percent_encoding::percent_decode(path.as_bytes())).unwrap();

    Ok(Some((version.to_owned(), method.to_owned(), path, query, headers)))
}
