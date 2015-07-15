extern crate http_server;

use std::io::{Read, Write};
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;

use http_server::http::{HttpServer, Request, Response};

fn handle(root: &Path, req: Request, mut res: Response) {
    let resource = root.join(req.path()).to_str().unwrap().to_owned();

    let metadata = match fs::metadata(&resource) {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };
    if metadata.is_file() {
        let mut f = File::open(&resource).unwrap();
        
        res.with_header("Connection", "close");
        res.with_header("Content-Type", "application/pdf");
        res.with_header("Content-Length", &metadata.len().to_string());

        res.start().unwrap();
        let mut buf: [u8; 1024] = [0; 1024];
        loop {
            match f.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    res.write(&buf).unwrap();
                },
                Err(_) => panic!("An error has ocurred")
            }
        }

        return;
    }

    let output = Command::new("ls")
        .arg(&resource)
        .output()
        .unwrap_or_else(|e| panic!(format!("Failed to list dir: {}", e)));

    let s: String;
    if output.status.success() {
        s = String::from_utf8_lossy(&output.stdout).as_ref().to_owned();
    }
    else {
        s = String::from_utf8_lossy(&output.stderr).as_ref().to_owned();
        panic!("rustc failed and stderr was:\n{}", s);
    }

    let mut path = req.path();
    if path.len() == 1 && path == "/" {
        path = "";
    }

    res.start().unwrap();
    res.write("<html><body><ul>".as_bytes()).unwrap();
    for file in s.split('\n') {
        if file.len() == 0 { continue }
        res.write(format!("<li><a href=\"{0}/{1}\">{1}</a></li>", path, file).as_bytes()).unwrap();
    }
    res.write("</ul></body></html>".as_bytes()).unwrap();
    res.flush().unwrap();
}

fn main() {
    let path = Path::new(".");

    let handler = move |req: Request, res: Response| {
        handle(&path, req, res);
    };

    let handler = Box::new(handler);
    let server: HttpServer = HttpServer::new("127.0.0.1:9999");
    server.start(handler);
}
