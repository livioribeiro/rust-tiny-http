use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::net::TcpStream;

use super::headers::Headers;
use super::query::Query;

pub fn parse_request(buf_reader: &mut BufReader<TcpStream>)
        -> io::Result<(String, String, String, Query, Headers)> {

    let mut line = String::new();

    buf_reader.read_line(&mut line).ok().expect("Failed to read request line");
    if line.is_empty() {
        return Err(Error::new(ErrorKind::Other, "bad request"));
    }

    let first_line: Vec<_> = line.split(' ').collect();
    let method = first_line[0];

    let version: Vec<_> = first_line[2].trim().split('/').collect();
    let version = version[1];

    let path_query: Vec<_> = first_line[1].split("?").collect();
    let path = path_query[0].to_owned();
    
    let query: Query;
    if path_query.len() > 1 {
        query = Query::from_str(path_query[1]);
    } else {
        query = Query::new();
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
            Err(error) => panic!("Error reading headers: {}", error),
        }
    }

    Ok((version.to_owned(), method.to_owned(), path.to_owned(), query, headers))
}
