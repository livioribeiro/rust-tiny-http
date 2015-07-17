use std::fmt::{self, Display, Formatter};
use std::io::{self, BufReader};
use std::net::{TcpStream, SocketAddr};

use super::headers::Headers;
use super::parser;
use super::query::Query;

#[derive(Debug)]
pub struct Request {
    http_version: String,
    method: String,
    scheme: String,
    path: String,
    query: Query,
    headers: Headers,
    content_length: Option<u64>,
    stream: BufReader<TcpStream>,
}

impl Request {
    pub fn from_stream(stream: TcpStream) -> io::Result<Request> {
        let mut buf_reader = BufReader::new(stream);
        let (version, method, path, query, headers) = try!(parser::parse_request(&mut buf_reader));

        Ok(Request {
            http_version: version.to_owned(),
            method: method.to_owned(),
            scheme: "http".to_owned(),
            path: path.to_owned(),
            query: query,
            content_length: None,
            headers: headers,
            stream: buf_reader,
        })
    }

    pub fn http_version(&self) -> &str {
        &self.http_version
    }

    pub fn method(&self) -> &str {
        self.method.as_ref()
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.stream.get_ref().local_addr().unwrap()
    }
}

impl Display for Request {
    fn fmt(&self, formatter: &mut Formatter ) -> Result<(), fmt::Error> {
        try!(writeln!(formatter, "{} {} {}", self.method, self.path, self.http_version));
        try!(write!(formatter, "Query: {:?}", self.query));
        try!(write!(formatter, "Headers: {}", self.headers));

        Ok(())
    }
}
