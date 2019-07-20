#![feature(drain_filter)]
#[macro_use]
extern crate bitflags;

use std::collections::LinkedList;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::time::{Duration, Instant};

mod epoll;

const TEAPOT: &[u8] = b"HTTP/1.1 418 I'm a teapot\r\n\r\n";

struct Tar<'run> {
    start: Instant,
    sock: &'run TcpStream,
}

fn main() {
    // setup listen with backlog of 128
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Unable to set nonblocking on listener");
    println!("Teapot starting on port 8080");

    // create an epoll data structure
    let epfd = match epoll::create() {
        Ok(epfd) => epfd,
        Err(e) => panic!(e),
    };
    // setup events on listener fd
    epoll::ctl(
        epfd,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        listener.as_raw_fd(),
        epoll::Event::new(
            epoll::Events::EPOLLIN | epoll::Events::EPOLLOUT,
            listener.as_raw_fd() as u64,
        ),
    );
    run(&listener, epfd);
}

fn run(listener: &TcpListener, epfd: RawFd) {
    let mut tarpit: LinkedList<Tar> = LinkedList::new();
    println!("running");
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // epoll wait, no conn attempted
                let mut events: [epoll::Event; 32] = Default::default();
                match epoll::wait(epfd, &mut events, 10000) {
                    Ok(nfds) => nfds,
                    Err(e) => panic!(e),
                };
                // attempt to empty tarpit
                tarpit.drain_filter(|tar| {
                    if Instant::now() - tar.start > Duration::new(10000, 0) {
                        println!("draining from tarpit after epoll timeout");
                        send_teapot(tar.sock);
                        return true;
                    }
                    return false;
                });
                continue;
            }
            Err(e) => panic!(e),
        };
        println!("stream is ready");
        stream
            .set_nonblocking(true)
            .expect("could not set stream nonblocking");

        // enqueue stream
        tarpit.push_back(Tar {
            start: Instant::now(),
            sock: &stream,
        });

        // drain tarpit
        tarpit.drain_filter(|tar| {
            if Instant::now() - tar.start > Duration::new(10000, 0) {
                println!("draining from tarpit after conn open");
                send_teapot(tar.sock);
                return true;
            }
            return false;
        });
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
