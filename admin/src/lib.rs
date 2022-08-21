pub mod ns;
pub mod tcp;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error interacting with the system: {0}")]
    System(#[from] nix::errno::Errno),
    #[error("error interacting with the system: {0}")]
    Io(#[from] std::io::Error),
    #[error("error in child proccess")]
    Child(),
}

pub type Result<T> = std::result::Result<T, crate::Error>;
