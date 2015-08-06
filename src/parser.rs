use std::error::Error;
use std::fmt;
use std::io::{self, Read, BufRead, BufReader, ErrorKind};
use regex::Regex;
use url::percent_encoding;

#[derive(Debug)]
pub enum ParserErrorKind {
    AbortError(ParserMethod),
    ExecutionError(Box<Error>),
}

impl fmt::Display for ParserErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserErrorKind::AbortError(ref e) => write!(f, "Parsing aborted from: {}", e),
            ParserErrorKind::ExecutionError(ref e) => write!(f, "Error parsing request: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct ParserError(ParserErrorKind);

impl ParserError {
    fn execution(error: Box<Error>) -> Result<(), Self> {
        Err(ParserError(ParserErrorKind::ExecutionError(error)))
    }

    fn abort(from_method: ParserMethod) -> Result<(), Self> {
        Err(ParserError(ParserErrorKind::AbortError(from_method)))
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        match self.0 {
            ParserErrorKind::AbortError(ref e) => {
                match *e {
                    ParserMethod::Method => "on_method",
                    ParserMethod::Url => "on_url",
                    ParserMethod::Query => "on_query",
                    ParserMethod::HttpVersion => "on_http_version",
                    ParserMethod::Status => "on_status",
                    ParserMethod::Header => "on_header",
                    ParserMethod::Body => "on_body",
                    ParserMethod::HeadersComplete => "on_headers_complete",
                    ParserMethod::MessageBegin => "on_message_begin",
                    ParserMethod::MessageComplete => "on_message_complete",
                }
            },
            ParserErrorKind::ExecutionError(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.0 {
            ParserErrorKind::AbortError(_) => None,
            ParserErrorKind::ExecutionError(ref e) => Some(&**e)
        }
    }
}

#[derive(Debug)]
pub enum ParserMethod {
    Method,
    Url,
    Query,
    HttpVersion,
    Status,
    Header,
    Body,
    HeadersComplete,
    MessageBegin,
    MessageComplete,
}

impl fmt::Display for ParserMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
}

pub struct Parser<'a, H: 'a> {
    handler: &'a mut H,
}

impl<'a, H: ParserHandler> Parser<'a, H> {
    pub fn request(handler: &'a mut H) -> Parser<'a, H> {
        Parser { handler: handler }
    }

    pub fn parse<R: Read>(&mut self, stream: &mut R) -> Result<(), ParserError> {
        let mut buf_reader = BufReader::new(stream);

        let mut request_line = String::new();
        let bytes_read = match buf_reader.read_line(&mut request_line) {
            Ok(bytes_read) => bytes_read,
            Err(error) => return ParserError::execution(Box::new(error)),
        };

        if bytes_read == 0 || request_line.is_empty() {
            return Ok(());
        }

        if !self.handler.on_message_begin() {
            return ParserError::abort(ParserMethod::MessageBegin);
        }

        let re = Regex::new(
            r"^(?P<method>[A-Z]*?) (?P<url>[^\?]+)(\?(?P<query>[^#]+))? HTTP/(?P<version>\d\.\d)\r\n$"
        ).unwrap();

        match re.captures(&request_line) {
            Some(cap) => {
                let method = cap.name("method").unwrap();
                if !self.handler.on_method(method) {
                    return ParserError::abort(ParserMethod::Method);
                }

                let url = percent_encoding::percent_decode(
                    cap.name("url").unwrap().as_bytes()
                );
                if !self.handler.on_url(String::from_utf8_lossy(&url).as_ref()) {
                    return ParserError::abort(ParserMethod::Url);
                }

                match cap.name("query") {
                    Some(query) => {
                        let query = percent_encoding::percent_decode(query.as_bytes());
                        if !self.handler.on_query(String::from_utf8_lossy(&query).as_ref()) {
                            return ParserError::abort(ParserMethod::Query);
                        }
                    }
                    None => {}
                }

                let version = cap.name("version").unwrap();
                if !self.handler.on_http_version(version) {

                }
            },
            None => {
                let error = io::Error::new(ErrorKind::InvalidInput, "Malformed Request");
                return ParserError::execution(Box::new(error));
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
                        return Ok(());
                    }
                },
                Err(e) => {
                    return ParserError::execution(Box::new(e));
                }
            }
        }

        Ok(())
    }
}
