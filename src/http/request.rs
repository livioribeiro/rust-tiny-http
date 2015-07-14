use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    method: String,
    path: String,
    query: HashMap<String, Vec<String>>,
    http_version: String,
    headers: HashMap<String, Vec<String>>,
    body: String,
}

pub struct RequestBuilder {
    method: String,
    path: String,
    query: HashMap<String, Vec<String>>,
    http_version: String,
    headers: HashMap<String, Vec<String>>,
    body: String,
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            method: String::new(),
            path: String::new(),
            query: HashMap::<String, Vec<String>>::new(),
            http_version: String::new(),
            headers: HashMap::<String, Vec<String>>::new(),
            body: String::new(),
        }
    }

    pub fn with_method(&mut self, method: &str) -> &mut Self {
        self.method = method.to_string();
        self
    }

    pub fn with_path(&mut self, path: &str) -> &mut Self {
        self.path = path.to_string();
        self
    }

    pub fn with_query(&mut self, name: &str, value: &str) -> &mut Self {
        {
            let mut query_vec = self.query.entry(name.to_string()).or_insert(Vec::new());
            query_vec.push(value.to_string());
        }
        self
    }

    pub fn with_http_version(&mut self, version: &str) -> &mut Self {
        self.http_version = version.to_string();
        self
    }

    pub fn with_header(&mut self, header: &str, value: &str) -> &mut Self {
        {
            let mut header_vec = self.headers.entry(header.to_string()).or_insert(Vec::new());
            header_vec.push(value.to_string());
        }
        self
    }

    pub fn with_body(&mut self, body: &str) -> &mut Self {
        self.body = body.to_string();
        self
    }

    pub fn build(self) -> Request {
        Request {
            method: self.method,
            path: self.path,
            query: self.query,
            http_version: self.http_version,
            headers: self.headers,
            body: self.body,
        }
    }
}
