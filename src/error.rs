use libc::c_int;

use crate::mount::MountErr;

#[derive(Debug)]
pub enum ContainerErr {
    Args(String),
    Bundle(String),
    IO(std::io::Error),
    Cgroup(String),
    State(String),
    Pipe(String),
    Fifo(String),
    Init(&'static str),
    Rlimit(String),
    IoPriority(String),
    InvalidNamespace(String),
    JoinNamespace(String),
    Clone(String),
    RootFs(String),
    Mount(MountErr),
    MountType(String),
    Options(String),
    Child((c_int, String)),
}

impl ContainerErr {
    pub fn invalid_args(msg: &str) -> Self {
        Self::Args(String::from(msg))
    }
}
