use admin::Result;
use nix::sched;

fn main() -> Result<()> {
    let defns = admin::open_ns(None)?;
    admin::create_ns("test")?;
    let newns = admin::open_ns(Some("test"))?;

    sched::setns(newns, nix::sched::CloneFlags::empty()).expect("failed to set ns");
    let listener = std::net::TcpListener::bind("127.0.0.1:8000").expect("failed to listen");
    sched::setns(defns, nix::sched::CloneFlags::empty()).expect("failed to set ns");

    for stream in listener.incoming() {
        admin::print_stream(stream.unwrap());
    }

    Ok(())
}
