use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::io::{self, BufReader};
use std::net::{TcpStream, SocketAddr};

use super::headers::Headers;
use super::parser;

#[derive(Debug)]
pub struct Query {
    data: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl Query {
    pub fn new() -> Query {
        Query {
            data: HashMap::<String, Vec<String>>::new(),
        }
    }

    pub fn parse(query_string: &str) -> Query {
        let mut query = Query::new();

        if query_string.trim().len() == 0 {
            return query;
        }

        for q in query_string.split("&") {
            let key_value: Vec<_> = q.split("=").collect();
            let key = key_value[0];
            let value = key_value[1];

            let mut query_vec = query.data.entry(key.to_string()).or_insert(Vec::new());
            query_vec.push(value.to_string());
        }

        query
    }

    pub fn get(&self, name: &str) -> Option<Vec<String>> {
        match self.data.get(name) {
            Some(values) => {
                if values.is_empty() {
                    None
                }
                else {
                    Some(values.clone())
                }
            },
            None => None
        }
    }

    pub fn add(&mut self, name: &str, value: &str) {
        let mut vec = self.data.entry(name.to_string()).or_insert(Vec::<String>::new());
        vec.push(value.to_string());
    }
}

impl Display for Query {
    fn fmt(&self, formatter: &mut Formatter ) -> Result<(), fmt::Error> {
        try!(writeln!(formatter, "{:?}", self.data));
        Ok(())
    }
}

#[derive(Debug)]
pub struct Request {
    http_version: String,
    method: String,
    scheme: String,
    path: String,
    query: Query,
    content_length: Option<u64>,
    headers: Headers,
    stream: BufReader<TcpStream>,
}

impl Request {
    pub fn create(stream: TcpStream) -> io::Result<Request> {
        let mut buf_reader = BufReader::new(stream);
        let (version, method, path, query, headers) = try!(parser::parse_request(&mut buf_reader));

        let query = match query {
            Some(q) => Query::parse(&q),
            None => Query::new(),
        };

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

    pub fn query(self) -> Query {
        self.query
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
