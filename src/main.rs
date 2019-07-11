use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use threadpool::ThreadPool;

const TEAPOT: &[u8] = b"HTTP/1.1 418 I'm a teapot\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let pool = ThreadPool::new(16);

    for stream in listener.incoming() {
        // gracefully handle bad listen
        let stream = match stream {
            Ok(stream) => stream,
            Err(_) => continue,
        };
        pool.execute(|| {
            handle(stream);
        });
    }
}

fn handle(mut stream: TcpStream) {
    match stream.write(TEAPOT) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
}
