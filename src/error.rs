#[derive(Debug)]
pub enum ContainerErr {
    Args(String),
    Bundle(String),
    Child(String),
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
}

impl ContainerErr {
    pub fn invalid_args(msg: &str) -> Self {
        Self::Args(String::from(msg))
    }
}
