#[macro_use]
extern crate bitflags;

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

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

    // use epoll_create and pass it to listener
    let epfd = match epoll::create() {
        Ok(epfd) => epfd,
        Err(e) => panic!(e),
    };
    let mut event = epoll::Event::new(
        epoll::Events::EPOLLIN | epoll::Events::EPOLLOUT,
        listener.as_raw_fd() as u64,
    );
    epoll::ctl(
        epfd,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        listener.as_raw_fd(),
        event,
    );
    run(&listener, epfd);
}

fn run(listener: &TcpListener, epfd: RawFd) {
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // epoll wait
                let mut events: [epoll::Event; 32] = Default::default();
                let nfds = match epoll::wait(epfd, &mut events, 10000) {
                    Ok(nfds) => nfds,
                    Err(e) => panic!(e),
                };
                continue;
            }
            Err(e) => panic!(e),
        };
        stream
            .set_nonblocking(true)
            .expect("could not set stream nonblocking");

        send_teapot(&stream);
    }
}

fn send_teapot(mut stream: &TcpStream) {
    match stream.write(TEAPOT) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }
    // shutdown to prevent TCP RST
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }
}
