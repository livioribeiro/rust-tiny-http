use std::error::Error;
use std::io::{self, Read, BufRead, BufReader, ErrorKind};
use regex::Regex;
use url;
use url::percent_encoding;

use super::headers::Headers;
use super::query::Query;

pub trait ParserHandler {
    fn on_url(&mut self, _url: &str) -> bool { true }
    fn on_status(&mut self, _status: u16) -> bool { true }
    fn on_header(&mut self, _field: &str, _values: Vec<&str>) -> bool { true }
    fn on_body(&mut self, _part: &[u8]) -> bool { true }
    fn on_headers_complete(&mut self) -> bool { true }
    fn on_message_begin(&mut self) -> bool { true }
    fn on_message_complete(&mut self) -> bool { true }
    fn on_error<E: Error>(&mut self, _error: E) { }
}

pub struct Parser<H> {
    handler: H,
    method: String,
    http_major: u16,
    http_minor: u16,
}

impl<H: ParserHandler> Parser<H> {
    pub fn request(handler: H) -> Parser<H> {
        Parser {
            handler: handler,
            method: "".to_owned(),
            http_major: 0_u16,
            http_minor: 0_u16,
        }
    }

    pub fn http_version(&self) -> (u16, u16) {
        (self.http_major, self.http_minor)
    }

    pub fn http_method(&self) -> &str {
        self.method.as_ref()
    }

    pub fn parse<R: Read>(&mut self, stream: &mut R) -> io::Result<bool> {
        let mut buf_reader = BufReader::new(stream);

        let mut request_line = String::new();
        let bytes_read = try!(buf_reader.read_line(&mut request_line));

        if bytes_read == 0 || request_line.is_empty() {
            return Ok(false);
        }

        self.handler.on_message_begin();

        let malformed_request_error = io::Error::new(ErrorKind::InvalidInput, "Malformed Request");

        let re = Regex::new(
            r"^(?P<method>[A-Z]*?) (?P<url>[^\?]+)(\?(?P<query>[^#]+))? HTTP/(?P<version>\d\.\d)\r\n$"
        ).unwrap();

        let (method, url, version, _query) = match re.captures(&request_line) {
            Some(cap) => {
                let method = cap.name("method").unwrap();
                let url = String::from_utf8_lossy(
                    &percent_encoding::percent_decode(
                        cap.name("url").unwrap().as_bytes()
                    )
                ).into_owned();
                let query = String::from_utf8_lossy(
                    &percent_encoding::percent_decode(
                        cap.name("query").unwrap().as_bytes()
                    )
                ).into_owned();
                let query = Query::from_str(&query);
                let version: Vec<&str> = cap.name("version").unwrap().split('.').collect();
                (method, url, version, query)
            },
            None => {
                self.handler.on_error(malformed_request_error);
                return Ok(false);
            },
        };

        self.method = method.to_owned();
        self.http_major = version[0].parse().unwrap();
        self.http_minor = version[1].parse().unwrap();

        if self.handler.on_url(&url) {
            return Ok(false);
        }

        // reading headers
        for line in buf_reader.lines() {
            match line {
                Ok(header_line) => {
                    // read an empty line
                    if header_line.trim().len() == 0 {
                        break;
                    }
                    let header: Vec<_> = header_line.split(": ").collect();
                    let field = header[0];
                    let values = header[1].split(',').collect();

                    self.handler.on_header(field, values);
                },
                Err(e) => return Err(e),
            }
        }

        Ok(true)
    }
}

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
