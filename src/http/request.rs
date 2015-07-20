use std::error::Error;
use std::io::{BufReader, Read};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use conduit::Request as ConduitRequest;
use conduit::Headers as ConduitHeaders;
use conduit::{self, Host, Method, Scheme};
use semver::Version;

use super::headers::Headers;
use super::parser;
use super::query::Query;

#[allow(dead_code)]
pub struct Request {
    http_version: Version,
    method: Method,
    scheme: Scheme,
    path: String,
    query: Query,
    headers: Headers,
    content_length: Option<u64>,
    stream: BufReader<TcpStream>,
    extensions: conduit::Extensions,
}

impl Request {
    pub fn from_stream(stream: TcpStream) -> Result<Request, Box<Error>> {
        let mut buf_reader = BufReader::new(stream);
        let (version, method, path, query, headers) = try!(parser::parse_request(&mut buf_reader));

        let mut content_length: Option<u64>;
        {
            let header: Option<Vec<&str>> = headers.find("Content-Length");
            content_length = match header {
                Some(l) => Some(try!(u64::from_str(l[0]))),
                None => None,
            };
        }

        Ok(Request {
            http_version: version,
            method: method,
            scheme: Scheme::Http,
            path: path,
            query: query,
            content_length: content_length,
            headers: headers,
            stream: buf_reader,
            extensions: conduit::Extensions::new(),
        })
    }
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
        Host::Socket(self.stream.get_ref().local_addr().unwrap())
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
        self.stream.get_ref().peer_addr().unwrap()
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
