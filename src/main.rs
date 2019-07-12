use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use threadpool::ThreadPool;

const TEAPOT: &[u8] = b"HTTP/1.1 418 I'm a teapot\r\n\r\n";

// Use a listen syscall and handle connections in a threadpool
fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let pool = ThreadPool::new(16);

    println!("Teapot starting on port 8080");

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
    // don't read from the socket to limit use of file descriptors
    match stream.write(TEAPOT) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
}
