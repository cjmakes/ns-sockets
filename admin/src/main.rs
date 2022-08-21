use admin::Result;
use admin::{ns, tcp};
use nix::sched;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

const NSNAME: &str = "test";

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let defns = ns::open_ns(None)?;

    ns::create_ns(NSNAME)?;
    info!(NSNAME, "created ns");

    let newns = ns::open_ns(Some("test"))?;

    sched::setns(newns, nix::sched::CloneFlags::empty()).expect("failed to set ns");
    info!(NSNAME, "in ns");

    // add addr to lo in ns
    // set lo up in ns
    let listener = std::net::TcpListener::bind("127.0.0.1:8000").expect("failed to listen");

    sched::setns(defns, nix::sched::CloneFlags::empty()).expect("failed to set ns");
    info!(NSNAME, "in default");

    for stream in listener.incoming() {
        tcp::print_stream(stream.unwrap());
    }

    Ok(())
}
