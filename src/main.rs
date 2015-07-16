extern crate http_server;
extern crate conduit_mime_types;

use std::env;
use std::io::{Read, Write};
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;

use conduit_mime_types::Types;
use http_server::http::{HttpServer, Request, Response};

fn handle(root: &Path, mimetypes: &Types, req: Request, mut res: Response) {
    let path = req.path().to_owned();
    let path = &path[1..path.len()];
    let resource = root.join(path).to_str().unwrap().to_owned();

    let metadata = match fs::metadata(&resource) {
        Ok(res) => res,
        Err(e) => { println!("{}", resource); panic!("{}", e); },
    };

    if metadata.is_file() {
        let mut f = File::open(&resource).unwrap();
        let mime = mimetypes.mime_for_path(Path::new(&resource));

        res.with_header("Connection", "close");
        res.with_header("Content-Type", mime);
        res.with_header("Content-Length", &metadata.len().to_string());

        res.start().unwrap();
        let mut buf: [u8; 4096] = [0; 4096];
        loop {
            match f.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    res.write(&buf[0..bytes_read]).unwrap();
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
    for name in s.split('\n') {
        if name.len() == 0 { continue }
        let mut name = name.to_owned();

        let metadata = fs::metadata(Path::new(&resource).join(&name)).unwrap();

        if metadata.is_dir() {
            name = format!("{}/", name);
        }

        res.write(format!("<li><a href=\"{0}/{1}\">{1}</a></li>", path, name).as_bytes()).unwrap();
    }
    res.write("</ul></body></html>".as_bytes()).unwrap();
    res.flush().unwrap();
}

fn main() {
    let path = env::current_dir().unwrap().as_path().to_owned();
    let mimetypes = Types::new().unwrap();

    let handler = move |req: Request, res: Response| {
        handle(&path, &mimetypes, req, res);
    };

    let handler = Box::new(handler);
    let server: HttpServer = HttpServer::new("127.0.0.1:9000");
    server.start(handler);
}
