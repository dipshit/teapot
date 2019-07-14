use libc::{fork, getpid};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener};

const TEAPOT: &[u8] = b"HTTP/1.1 418 I'm a teapot\r\n\r\n";

// Use a listen syscall and handle connections in a threadpool
fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set listener non-blocking");
    println!("Teapot starting on port 8080");

    unsafe {
        let parent_pid = getpid();
        println!("parent is {}", parent_pid);
        for _ in 1..16 {
            if getpid() == parent_pid {
                fork();
            }
        }
    }
    run(&listener);
}

// have all procs epoll_create and block on it
fn run(listener: &TcpListener) {
    unsafe {
        println!("pid {} ready", getpid());
    }
    // accept in a loop
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            // drop connection if out of file descriptors
            Err(_) => continue,
        };
        stream
            .set_nonblocking(true)
            .expect("Cannot set stream non-blocking");

        // wait until socket is done being written to before writing
        // this helps avoid TCP RST if we read non blocking
        let mut buffer = [0];
        match stream.read(&mut buffer) {
            Ok(_) => (),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("Read would block: {}", e);
                continue;
            }
            Err(e) => println!("{}", e),
        }

        match stream.write(TEAPOT) {
            Ok(_) => (),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("Write would block: {}", e);
                continue;
            }
            Err(e) => println!("{}", e),
        }

        // shutdown to prevent TCP RST
        match stream.shutdown(Shutdown::Both) {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }
    } // close socket
}
