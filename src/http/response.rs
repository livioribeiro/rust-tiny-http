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
    pub fn new(version: &str, status: i32, status_text: &str, stream: TcpStream) -> Response {
        Response {
            http_version: version.to_string(),
            status: status,
            status_text: status_text.to_string(),
            headers: Headers::new(),
            stream: stream,
            headers_written: false,
        }
    }

    pub fn with_header(&mut self, name: &'static str, value: &str) -> &Self {
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

        let status_line = format!("{} {} {}\r\n", self.http_version, self.status, self.status_text);
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
