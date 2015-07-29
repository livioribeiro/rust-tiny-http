use std::io::{self, BufReader, Read};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use super::headers::Headers;
use super::parser;
use super::query::Query;

pub struct RequestStream {
    stream: BufReader<TcpStream>,
}

impl RequestStream {
    pub fn from_stream(stream: &TcpStream) -> io::Result<Self> {
        let stream = try!(stream.try_clone());
        let buf_reader = BufReader::new(stream);

        Ok(RequestStream {
            stream: buf_reader,
        })
    }

    pub fn requests(&mut self) -> RequestStreamIterMut {
        RequestStreamIterMut { request_stream: self }
    }

    fn next_request(&mut self) -> Option<Request> {
        let (version, method, path, query, headers) = match parser::parse_request(&mut self.stream).unwrap() {
            Some(result) => result,
            None => return None,
        };

        let mut content_length: Option<u64>;
        {
            let header: Option<Vec<&str>> = headers.find("Content-Length");
            content_length = header.map(|line| u64::from_str(line[0]).unwrap());
        }

        Some(Request {
            http_version: version,
            method: method,
            scheme: "Http".to_owned(),
            path: path,
            query: query,
            content_length: content_length,
            headers: headers,
            stream: self.stream.get_ref().try_clone().unwrap(),
        })
    }
}

pub struct RequestStreamIterMut<'a> {
    request_stream: &'a mut RequestStream,
}

impl<'a> Iterator for RequestStreamIterMut<'a> {
    type Item = Request;

    fn next(&mut self) -> Option<Self::Item> {
        self.request_stream.next_request()
    }
}

#[allow(dead_code)]
pub struct Request {
    http_version: String,
    method: String,
    scheme: String,
    path: String,
    query: Option<Query>,
    headers: Headers,
    content_length: Option<u64>,
    stream: TcpStream,
}

impl Request {
    pub fn http_version(&self) -> &str {
        &self.http_version
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host(&self) -> SocketAddr {
        self.stream.local_addr().unwrap()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> &Option<Query> {
        &self.query
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn body<'a>(&'a mut self) -> &'a mut Read {
        &mut self.stream
    }
}
