use std::error::Error;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::net::TcpStream;

use conduit::Method;
use regex::Regex;
use semver::Version;

use super::headers::Headers;
use super::query::Query;

pub fn parse_request(buf_reader: &mut BufReader<TcpStream>)
        -> Result<Option<(Version, Method, String, Query, Headers)>, Box<Error>> {

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
            let query = match cap.name("query") {
                Some(q) => Query::from_str(q),
                None => Query::new(),
            };
            (method, path, version, query)
        },
        None => return Err(Box::new(malformed_request_error)),
    };

    let method = match method {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        "HEAD" => Method::Head,
        "CONNECT" => Method::Connect,
        "OPTIONS" => Method::Options,
        "TRACE" => Method::Trace,
        "PATCH" => Method::Patch,
        "PURGE" => Method::Purge,
        _ => Method::Other("UNKNOWN"),
    };

    let version = match version {
        "1.0" => Version::parse("1.0.0").unwrap(),
        "1.1" => Version::parse("1.1.0").unwrap(),
        "2.0" => Version::parse("2.0.0").unwrap(),
        _ => {
            let mut v = version.to_string();
            v.push_str(".0");
            try!(Version::parse(v.as_ref()))
        },
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

    Ok(Some((version, method, path.to_owned(), query, headers)))
}
