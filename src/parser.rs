use std::error::Error;
use std::io::{self, Read, BufRead, BufReader, ErrorKind};
use regex::Regex;
use url::percent_encoding;

pub trait ParserHandler {
    fn on_method(&mut self, _method: &str) -> bool { true }
    fn on_url(&mut self, _url: &str) -> bool { true }
    fn on_query(&mut self, _query: &str) -> bool { true }
    fn on_http_version(&mut self, _version: &str) -> bool { true }
    fn on_status(&mut self, _status: u16) -> bool { true }
    fn on_header(&mut self, _field: &str, _values: Vec<&str>) -> bool { true }
    fn on_body(&mut self, _part: &[u8]) -> bool { true }
    fn on_headers_complete(&mut self) -> bool { true }
    fn on_message_begin(&mut self) -> bool { true }
    fn on_message_complete(&mut self) -> bool { true }
    fn on_error<E: Error>(&mut self, _error: &E) { }
}

pub struct Parser<'a, H: 'a> {
    handler: &'a mut H,
}

impl<'a, H: ParserHandler> Parser<'a, H> {
    pub fn request(handler: &'a mut H) -> Parser<'a, H> {
        Parser { handler: handler }
    }

    pub fn parse<R: Read>(&mut self, stream: &mut R) -> io::Result<bool> {
        let mut buf_reader = BufReader::new(stream);

        let mut request_line = String::new();
        let bytes_read = try!(buf_reader.read_line(&mut request_line));

        if bytes_read == 0 || request_line.is_empty() {
            return Ok(false);
        }

        if !self.handler.on_message_begin() {
            return Ok(false);
        }

        let malformed_request_error = io::Error::new(ErrorKind::InvalidInput, "Malformed Request");

        let re = Regex::new(
            r"^(?P<method>[A-Z]*?) (?P<url>[^\?]+)(\?(?P<query>[^#]+))? HTTP/(?P<version>\d\.\d)\r\n$"
        ).unwrap();

        match re.captures(&request_line) {
            Some(cap) => {
                let method = cap.name("method").unwrap();
                if !self.handler.on_method(method) {
                    return Ok(false);
                }

                let url = percent_encoding::percent_decode(
                    cap.name("url").unwrap().as_bytes()
                );
                if !self.handler.on_url(String::from_utf8_lossy(&url).as_ref()) {
                    return Ok(false);
                }

                cap.name("query").map(|q| {
                    let query = percent_encoding::percent_decode(q.as_bytes());
                    self.handler.on_query(String::from_utf8_lossy(&query).as_ref())
                });

                let version = cap.name("version").unwrap();
                if !self.handler.on_http_version(version) {

                }
            },
            None => {
                self.handler.on_error(&malformed_request_error);
                return Ok(false);
            },
        };

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

                    if !self.handler.on_header(field, values) {
                        return Ok(false);
                    }
                },
                Err(e) => {
                    self.handler.on_error(&e);
                    return Err(e);
                }
            }
        }

        Ok(true)
    }
}
