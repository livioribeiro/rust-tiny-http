# tiny-http-server
[![Build Status](https://travis-ci.org/livioribeiro/rust-tiny-http.svg?branch=master)](https://travis-ci.org/livioribeiro/rust-tiny-http)

Small http server written in rust

This is an experiment I started to learn the Rust language.

I decided to make an Http server because it is a good challenge and I had a lot of fun building it.

To start the server, just clone this repo and run `$ cargo run` and the it will start listening at port 9000. To change the address the server will listen and enable directory listing, run `$ cargo run -- -a 127.0.0.1:80 -d` (or `$ cargo run -- --addr=127.0.0.1:80 --dir`)

Currently, you can only change the server root by editing `main.rs`.

```rust

fn main() {
    // ...

    // Edit here to change the server root
    let path = env::home_dir().unwrap();

    // ...
}
```
