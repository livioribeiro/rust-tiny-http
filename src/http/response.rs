use std::io;
use std::io::Write;
use std::net::TcpStream;

use super::headers::Headers;

#[derive(Debug)]
pub struct Response {
    http_version: String,
    status: i32,
    status_text: String,
    headers: Headers,
    stream: TcpStream,
    headers_written: bool,
}

impl Response {
    pub fn new(stream: TcpStream) -> Response {
        Response {
            http_version: "1.0".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            headers: Headers::new(),
            stream: stream,
            headers_written: false,
        }
    }

    pub fn http_version(&self) -> &str {
        self.http_version.as_ref()
    }

    pub fn http_version_text(&self) -> String {
        let mut ver = "HTTP/".to_string();
        for c in self.http_version.chars() {
            ver.push(c);
        }
        ver
    }

    pub fn with_http_version(&mut self, version: &str) -> &mut Self {
        if self.headers_written {
            panic!("Cannot write header to started response")
        }

        self.http_version = version.to_string();
        self
    }

    pub fn status(&self) -> (i32, &str) {
        (self.status, self.status_text.as_ref())
    }

    pub fn with_status(&mut self, status: i32, status_text: &str) -> &mut Self {
        if self.headers_written {
            panic!("Cannot write header to started response")
        }

        self.status = status;
        self.status_text = status_text.to_string();
        self
    }

    pub fn with_header(&mut self, name: &'static str, value: &str) -> &mut Self {
        if self.headers_written {
            panic!("Cannot write header to started response")
        }
        self.headers.insert(name, value);
        self
    }

    pub fn start(&mut self) -> Result<&Self, io::Error> {
        if self.headers_written {
            panic!("Response already started");
        }

        self.headers_written = true;

        let status_line = format!("HTTP/{} {} {}\r\n", self.http_version, self.status, self.status_text);
        try!(self.write(status_line.as_bytes()));

        let headers = self.headers.clone();
        try!(self.write(format!("{}", headers).as_bytes()));
        try!(self.write("\r\n\r\n".as_bytes()));

        Ok(self)
    }
}

impl Write for Response {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        if !self.headers_written {
            panic!("Headers not written");
        }
        try!(self.stream.write(buf));
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        if !self.headers_written {
            panic!("Headers not written");
        }
        self.stream.flush()
    }
}
