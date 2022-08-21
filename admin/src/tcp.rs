use nix::sys::socket;

use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::os::unix::io::RawFd;

use io_uring::{opcode, types, IoUring};

use crate::Result;

pub fn print_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);
}

fn accept1(fd: RawFd) -> Result<i32> {
    let mut ring = IoUring::new(1024)?;

    let mut sa: libc::sockaddr = unsafe { std::mem::zeroed() };
    let mut sl: libc::socklen_t = 0;
    let acc_sqe = opcode::Accept::new(types::Fd(fd), &mut sa, &mut sl).build();

    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        ring.submission().push(&acc_sqe).unwrap();
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    Ok(cqe.result())
}

fn create_listener() -> Result<RawFd> {
    let sock_addr = socket::SockaddrIn::new(127, 0, 0, 1, 8000);

    let ssock = socket::socket(
        socket::AddressFamily::Inet,
        socket::SockType::Stream,
        socket::SockFlag::empty(),
        None,
    )
    .expect("failed to get sock");

    socket::bind(ssock, &sock_addr).expect("failed to bind");

    Ok(ssock)

    // send socket

    // send socket fd to parent in main ns
    // install sk_lookup
    // open a new connection
    // proxy
}
