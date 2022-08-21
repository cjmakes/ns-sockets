use crate::{Error, Result};

use std::os::unix::io::RawFd;
use std::path::Path;

use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::sys::wait::WaitStatus;
use nix::{mount, sched};

use tracing::{error, instrument, span, Level};

#[instrument]
pub fn open_ns(name: Option<&str>) -> Result<RawFd> {
    let span = span!(Level::TRACE, "opening ns", ?name);
    let _enter = span.enter();

    let path = match name {
        None => "/proc/self/ns/net".to_string(),
        Some(n) => format!("/var/run/netns/{}", n),
    };

    let fd = nix::fcntl::open(
        Path::new(&path),
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?;

    Ok(fd)
}

#[instrument]
pub fn create_ns(name: &str) -> Result<()> {
    let ref mut stack = [0u8; 1024];
    let child_pid = sched::clone(
        Box::new(|| setup_ns(name)),
        stack,
        sched::CloneFlags::CLONE_NEWNET,
        Some(nix::sys::signal::Signal::SIGCHLD as i32),
    )?;

    match nix::sys::wait::waitpid(child_pid, None)? {
        WaitStatus::Exited(_, rc) if rc != 0 => {
            error!("child failed");
            Err(Error::Child())
        }
        _ => Ok(()),
    }
}

#[instrument]
fn setup_ns(name: &str) -> isize {
    if let Err(error) = pin_ns(name) {
        error!("failed to pin: {}", error);
        return -1;
    }

    0
}

#[instrument]
fn pin_ns(name: &str) -> Result<()> {
    std::fs::create_dir_all("/var/run/netns/")?;
    std::fs::File::create(format!("/var/run/netns/{name}"))?;

    mount::mount::<Path, Path, Path, Path>(
        Some(Path::new("/proc/self/ns/net")),
        Path::new(&format!("/var/run/netns/{name}")),
        None,
        mount::MsFlags::MS_BIND,
        None,
    )?;

    Ok(())
}
