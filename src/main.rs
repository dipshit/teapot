use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use threadpool::ThreadPool;

const TEAPOT: &[u8] = b"HTTP/1.1 418 I'm a teapot\r\n\r\n";
const NOTFOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";
const UNAVAILABLE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\n\r\n";

const GET: &[u8] = b"GET / HTTP/1.1\r\n";

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
    } // socket closed
}

fn handle(mut stream: TcpStream) {
    // only read the first 512 bytes
    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(_) => (),
        // respond 503 if out of file descriptors
        Err(_) => {
            send(UNAVAILABLE, stream);
            return;
        }
    }
    if buffer.starts_with(GET) {
        return send(TEAPOT, stream);
    }
    // issue Not Found instead of 400 Bad Request so we don't have
    // to parse the request
    send(NOTFOUND, stream);
}

fn send(response: &[u8], mut stream: TcpStream) {
    match stream.write(response) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
    // prevent TCP RST if client sent req longer than 512 bytes
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
}
