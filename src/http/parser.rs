use std::error::Error;
use std::io::{self, Read, BufRead, BufReader};
use std::net::TcpStream;

use super::request::Request;
use super::headers::Headers;

pub fn parse_request<R: Read>(buf_reader: &mut BufReader<R>)
        -> Result<(String, String, String, Option<String>, Headers), io::Error> {
    let mut line = String::new();

    try!(buf_reader.read_line(&mut line));
    if line.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "bad request"));
    }

    let first_line: Vec<_> = line.split(' ').collect();
    let method = first_line[0];

    let version: Vec<_> = first_line[2].trim().split('/').collect();
    let version = version[1];

    let path_query: Vec<_> = first_line[1].split("?").collect();
    let path = path_query[0];
    let query: Option<String>;
    if path_query.len() > 1 {
        query = Some(path_query[1].to_owned());
    }
    else {
        query = None;
    }

    let mut headers = Headers::new();

    for line in buf_reader.lines() {
        match line {
            Ok(l) => {
                if l.trim().len() == 0 {
                    break;
                }
                headers.parse(&l);
            },
            Err(e) => return Err(e)
        }
    }

    Ok((version.to_owned(), method.to_owned(), path.to_owned(), query, headers))
}
