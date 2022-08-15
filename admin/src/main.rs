use nix::fcntl::OFlag;
use nix::sched;
use nix::sys::socket;
use nix::sys::stat::Mode;
use std::os::unix::io::RawFd;

use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let defns = open_ns(None);
    let newns = open_ns(Some("test"));

    sched::setns(newns, nix::sched::CloneFlags::empty()).expect("failed to set ns");
    let listener = TcpListener::bind("127.0.0.1:8000").expect("failed to listen");
    sched::setns(defns, nix::sched::CloneFlags::empty()).expect("failed to set ns");

    for stream in listener.incoming() {
        print_stream(stream.unwrap());
    }

    Ok(())
}

use std::io::{prelude::*, BufReader};

fn open_ns(name: Option<&str>) -> i32 {
    let path = match name {
        None => format!("/proc/{}/ns/net", std::process::id()),
        Some(n) => format!("/var/run/netns/{}", n),
    };
    nix::fcntl::open(
        Path::new(&path),
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect("failed to open")
}

fn print_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);
}

fn create_ns() {
    // create new net ns
    let ref mut stack = [0u8; 1024];
    sched::clone(
        Box::new(|| setup_ns()),
        stack,
        sched::CloneFlags::CLONE_NEWNET,
        None,
    )
    .expect("failed to clone");
}

fn setup_ns() -> isize {
    create_listener().unwrap();
    install_sklookup();
    0
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

fn install_sklookup() {}
