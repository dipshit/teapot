#[macro_use]
extern crate bitflags;

use libc::{fork, getpid};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
mod epoll;

const TEAPOT: &[u8] = b"HTTP/1.1 418 I'm a teapot\r\n\r\n";

// Use a listen syscall and handle connections in a threadpool
fn main() {
    // backlog is 128 by default
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Unable to set nonblocking on listener");
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
    // use epoll_create and pass it to listener
    run(&listener);
}

fn run(listener: &TcpListener) {
    unsafe {
        println!("pid {} ready", getpid());
    }
    // create socket and listen with a backlog queue set
    // epoll_ctl add socket with raw fd
    //loop {
    // epoll_wait for data
    // get numfds
    // for numfds times
    //   if this iter is the sockfd, accept new_fd and epoll_ctl add it
    //   else it is a stream. send teapot and epoll_ctl it
    //}
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => respond(&mut s),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // epoll_wait until fd of listener is ready
                // use syscall and listener as raw fd
                continue;
            }
            Err(e) => println!("{}", e),
        }
    } // close socket
}

// new data is available somewhere
// call epoll_wait and respond to all ready fds
fn respond(stream: &mut TcpStream) {
    stream
        .set_nonblocking(true)
        .expect("Unable to set nonblocking on stream");

    // block until data is ready to avoid TCP RST
    let mut buffer = [0];
    match stream.read(&mut buffer) {
        Ok(_) => (),
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            // epoll_wait until fd of stream is ready to read from
            println!("read would block: {}", e);
        }
        Err(e) => println!("{}", e),
    }

    match stream.write(TEAPOT) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    // shutdown to prevent TCP RST
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
}
