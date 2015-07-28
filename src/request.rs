use std::io::{self, BufReader, Read};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use conduit::Request as ConduitRequest;
use conduit::Headers as ConduitHeaders;
use conduit::{self, Host, Method, Scheme};
use semver::Version;

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
            scheme: Scheme::Http,
            path: path,
            query: query,
            content_length: content_length,
            headers: headers,
            extensions: conduit::Extensions::new(),
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
    http_version: Version,
    method: Method,
    scheme: Scheme,
    path: String,
    query: Query,
    headers: Headers,
    content_length: Option<u64>,
    extensions: conduit::Extensions,
    stream: TcpStream,
}

impl ConduitRequest for Request {
    fn http_version(&self) -> Version {
        Version::parse("1.0.0").unwrap()
    }

    fn conduit_version(&self) -> Version {
        Version::parse("1.0.0").unwrap()
    }

    fn method(&self) -> Method {
        Method::Get
    }

    fn scheme(&self) -> Scheme {
        Scheme::Http
    }

    fn host<'a>(&'a self) -> Host<'a> {
        Host::Socket(self.stream.local_addr().unwrap())
    }

    fn virtual_root<'a>(&'a self) -> Option<&'a str> {
        None
    }

    fn path<'a>(&'a self) -> &'a str {
        self.path.as_ref()
    }

    fn query_string<'a>(&'a self) -> Option<&'a str> {
        self.query.query_string()
    }

    fn remote_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    fn headers<'a>(&'a self) -> &'a conduit::Headers {
        &self.headers
    }

    fn body<'a>(&'a mut self) -> &'a mut Read {
        &mut self.stream
    }

    fn extensions<'a>(&'a self) -> &'a conduit::Extensions {
        &self.extensions
    }

    fn mut_extensions<'a>(&'a mut self) -> &'a mut conduit::Extensions {
        &mut self.extensions
    }
}
