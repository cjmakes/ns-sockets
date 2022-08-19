use nix::fcntl::OFlag;
use nix::sys::socket;
use nix::sys::stat::Mode;
use nix::{mount, sched};

use std::error::Error;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::os::unix::io::RawFd;
use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn open_ns(name: Option<&str>) -> Result<RawFd> {
    let path = match name {
        None => "/proc/self/ns/net".to_string(),
        Some(n) => format!("/var/run/netns/{}", n),
    };

    Ok(nix::fcntl::open(
        Path::new(&path),
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect(&format!("failed to open: {}", path)))
}

pub fn print_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);
}

pub fn create_ns(name: &str) -> Result<()> {
    let ref mut stack = [0u8; 1024];
    let child_pid = sched::clone(
        Box::new(|| setup_ns(name)),
        stack,
        sched::CloneFlags::CLONE_NEWNET,
        Some(nix::sys::signal::Signal::SIGCHLD as i32),
    )
    .expect("failed to clone");

    nix::sys::wait::waitpid(child_pid, None)?;

    Ok(())
}

fn setup_ns(name: &str) -> isize {
    pin_ns(name);
    0
}

fn pin_ns(name: &str) -> isize {
    std::fs::create_dir_all("/var/run/netns/").expect("failed to make dir");
    std::fs::File::create(format!("/var/run/netns/{name}")).expect("failed to make ns file");

    // TODO create files
    if let Err(e) = mount::mount::<Path, Path, Path, Path>(
        Some(Path::new("/proc/self/ns/net")),
        Path::new(&format!("/var/run/netns/{name}")),
        None,
        mount::MsFlags::MS_BIND,
        None,
    ) {
        eprintln!("mount failed {e}");
        return -1;
    }
    return 0;
}

use io_uring::{opcode, types, IoUring};

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
