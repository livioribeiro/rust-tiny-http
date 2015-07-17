use std::any::Any;
use std::fs::{self, File, Metadata};
use std::io::{self, Write, Error, ErrorKind};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::Command;

use conduit_mime_types::Types;

use ::{Request, Response};

pub struct FileMode;
pub struct DirectoryMode;

pub trait Handler {
    fn handle(&self, req: &mut Request, res: &mut Response);
}

#[derive(Debug)]
pub struct ServerHandler<M: Any> {
    root: PathBuf,
    mimetypes: Types,
    _kind: PhantomData<M>,
}

impl<M: Any> ServerHandler<M> {
    pub fn new(root: &PathBuf) -> ServerHandler<M> {
        let mimetypes = match Types::new() {
            Ok(types) => types,
            Err(error) => panic!(error),
        };

        ServerHandler {
            root: root.to_owned(),
            mimetypes: mimetypes,
            _kind: PhantomData
        }
    }

    fn get_resource_and_metadata(&self, req: &Request)
            -> Result<(PathBuf, Metadata), Error> {

        let path = req.path();
        let path = &path[1..path.len()];

        let resource = self.root.join(path);
        let metadata = try!(fs::metadata(&resource));

        Ok((resource, metadata))
    }

    fn send_file(&self, resource: PathBuf, metadata: Metadata, res: &mut Response) {
        let mut f = File::open(&resource).unwrap();
        let mime = self.mimetypes.mime_for_path(Path::new(&resource));

        res.with_header("Connection", "close")
            .with_header("Content-Type", mime)
            .with_header("Content-Length", &metadata.len().to_string());

        let res = res.start().unwrap();
        io::copy(&mut f, res).unwrap();

        // let mut buf: [u8; 4096] = [0; 4096];
        // loop {
        //     match f.read(&mut buf) {
        //         Ok(bytes_read) => {
        //             if bytes_read == 0 {
        //                 break;
        //             }
        //             res.write(&buf[0..bytes_read]).unwrap();
        //         },
        //         Err(e) => panic!(e)
        //     }
        // }
    }

    fn send_not_found(&self, res: &mut Response) {
        res.with_status(404, "Not Found");
        let res = res.start().unwrap();
        res.write("404 - Not Found".as_bytes()).unwrap();
        res.flush().ok().expect("Failed to send error response");
    }

    fn send_error(&self, res: &mut Response) {
        res.with_status(404, "Not Found");
        let res = res.start().unwrap();
        res.write("404 - Not Found".as_bytes()).unwrap();
        res.flush().ok().expect("Failed to send error response");
    }
}

impl Handler for ServerHandler<FileMode> {
    fn handle(&self, req: &mut Request, res: &mut Response) {
        let (resource, metadata) = match self.get_resource_and_metadata(req) {
            Ok(result) => result,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    self.send_not_found(res);
                } else {
                    self.send_error(res);
                }
                return;
            }
        };

        if !metadata.is_file() {
            self.send_not_found(res);
            return;
        }

        self.send_file(resource, metadata, res);
    }
}

impl Handler for ServerHandler<DirectoryMode> {
    fn handle(&self, req: &mut Request, res: &mut Response) {
        let (resource, metadata) = match self.get_resource_and_metadata(req) {
            Ok(result) => result,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    self.send_not_found(res);
                } else {
                    self.send_error(res);
                }
                return;
            }
        };

        if metadata.is_file() {
            self.send_file(resource, metadata, res);
            return;
        }

        let output = Command::new("ls")
            .arg(&resource)
            .output()
            .unwrap_or_else(|e| panic!(format!("Failed to list dir: {}", e)));

        let s: String;
        if output.status.success() {
            s = String::from_utf8_lossy(&output.stdout).as_ref().to_owned();
        } else {
            s = String::from_utf8_lossy(&output.stderr).as_ref().to_owned();
            panic!("rustc failed and stderr was:\n{}", s);
        }

        let mut path = req.path();
        if path.len() == 1 && path == "/" {
            path = "";
        }

        res.with_header("Content-Type", "text/html");

        let res = res.start().unwrap();

        res.write("<html><body><ul>".as_bytes()).unwrap();
        for name in s.split('\n') {
            if name.len() == 0 { continue }
            let mut name = name.to_owned();

            let metadata = fs::metadata(Path::new(&resource).join(&name)).unwrap();

            if metadata.is_dir() {
                name = format!("{}/", name);
            }

            res.write(format!("<li><a href=\"{0}{1}\">{1}</a></li>", path, name).as_bytes()).unwrap();
        }
        res.write("</ul></body></html>".as_bytes()).unwrap();
        res.flush().unwrap();
    }
}
