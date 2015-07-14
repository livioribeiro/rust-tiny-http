use std::collections::HashMap;
use std::net::TcpStream;

use super::headers::Headers;

#[derive(Debug)]
pub struct Query {
    data: HashMap<String, Vec<String>>,
}

impl Query {
    pub fn new() -> Query {
        Query {
            data: HashMap::<String, Vec<String>>::new(),
        }
    }

    pub fn parse(query_string: &str) -> Query{
        let mut query = Query::new();

        for q in query_string.split("&") {
            let key_value: Vec<_> = q.split("=").collect();
            let key = key_value[0];
            let value = key_value[1];

            let mut query_vec = query.data.entry(key.to_string()).or_insert(Vec::new());
            query_vec.push(value.to_string());
        }

        query
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        match self.data.get(name) {
            Some(values) => {
                if values.is_empty() {
                    None
                }
                else {
                    Some(&values[0])
                }
            },
            None => None
        }
    }

    pub fn get_default(&self, name: &str, default: String) -> String {
        let default = default.to_string();
        match self.data.get(name) {
            Some(values) => {
                if values.is_empty() {
                    default
                }
                else {
                    values[0].clone()
                }
            },
            None => default
        }
    }

    pub fn insert(&mut self, name: &str, value: &str) {
        let mut vec = self.data.entry(name.to_string()).or_insert(Vec::<String>::new());
        vec.push(value.to_string());
    }
}

#[derive(Debug)]
pub struct Request {
    method: String,
    path: String,
    query: Query,
    http_version: String,
    headers: Headers,
    body_stream: TcpStream,
}

impl Request {
    pub fn new(method: &str,
               path: &str,
               query: &str,
               version: &str,
               headers: Headers,
               body: TcpStream) -> Request {

        Request {
            method: method.to_string(),
            path: path.to_string(),
            query: Query::parse(query),
            http_version: version.to_string(),
            headers: headers,
            body_stream: body
        }
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }
}
