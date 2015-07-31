use std::error::Error;
use std::io::{self, Read, BufRead, BufReader, ErrorKind};
use regex::Regex;
use url;
use url::percent_encoding;

use super::headers::Headers;
use super::query::Query;

pub fn parse_request<T: Read>(stream: T)
        -> Result<Option<(String, String, Vec<String>, Option<Query>, Headers)>, Box<Error>> {

    let mut buf_reader = BufReader::new(stream);

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

    let path = path.iter().map(|i| {
        String::from_utf8_lossy(&percent_encoding::percent_decode(i.as_bytes())).into_owned()
    }).collect();

    Ok(Some((version.to_owned(), method.to_owned(), path, query, headers)))
}
