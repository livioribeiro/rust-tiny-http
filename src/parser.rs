use std::error::Error;
use std::io::{self, Read, BufRead, BufReader, ErrorKind};
use regex::Regex;
use url::percent_encoding;

pub trait ParserHandler {
    fn on_method(&mut self, _method: &str) -> Result<(), Box<Error>> { Ok(()) }
    fn on_url(&mut self, _url: &str) -> Result<(), Box<Error>> { Ok(()) }
    fn on_query(&mut self, _query: &str) -> Result<(), Box<Error>> { Ok(()) }
    fn on_http_version(&mut self, _version: &str) -> Result<(), Box<Error>> { Ok(()) }
    fn on_status(&mut self, _status: u16) -> Result<(), Box<Error>> { Ok(()) }
    fn on_header(&mut self, _field: &str, _values: Vec<&str>) -> Result<(), Box<Error>> { Ok(()) }
    fn on_body(&mut self, _part: &[u8]) -> Result<(), Box<Error>> { Ok(()) }
    fn on_headers_complete(&mut self) -> Result<(), Box<Error>> { Ok(()) }
    fn on_message_begin(&mut self) -> Result<(), Box<Error>> { Ok(()) }
    fn on_message_complete(&mut self) -> Result<(), Box<Error>> { Ok(()) }
}

pub struct Parser<'a, H: 'a> {
    handler: &'a mut H,
}

impl<'a, H: ParserHandler> Parser<'a, H> {
    pub fn request(handler: &'a mut H) -> Parser<'a, H> {
        Parser { handler: handler }
    }

    pub fn parse<R: Read>(&mut self, stream: &mut R) -> Result<(), Box<Error>> {
        let mut buf_reader = BufReader::new(stream);

        let mut request_line = String::new();
        let bytes_read = try!(buf_reader.read_line(&mut request_line));

        if bytes_read == 0 || request_line.is_empty() {
            return Ok(());
        }

        try!(self.handler.on_message_begin());

        let re = Regex::new(
            r"^(?P<method>[A-Z]*?) (?P<url>[^\?]+)(\?(?P<query>[^#]+))? HTTP/(?P<version>\d\.\d)\r\n$"
        ).unwrap();

        match re.captures(&request_line) {
            Some(cap) => {
                let method = cap.name("method").unwrap();
                try!(self.handler.on_method(method));

                let url = percent_encoding::lossy_utf8_percent_decode(
                    cap.name("url").unwrap().as_bytes()
                );
                try!(self.handler.on_url(&url));

                match cap.name("query") {
                    Some(query) => {
                        let query = percent_encoding::lossy_utf8_percent_decode(query.as_bytes());
                        try!(self.handler.on_query(&query));
                    }
                    None => {}
                }

                let version = cap.name("version").unwrap();
                try!(self.handler.on_http_version(version));
            },
            None => {
                let error = io::Error::new(ErrorKind::InvalidInput, "Malformed Request");
                return Err(Box::new(error));
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

                    try!(self.handler.on_header(field, values));
                },
                Err(e) => {
                    return Err(Box::new(e));
                }
            }
        }

        Ok(())
    }
}
