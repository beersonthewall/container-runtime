#[derive(Debug)]
pub enum ContainerErr {
    Args(String),
    Bundle(String),
    Child(String),
    IO(std::io::Error),
}

impl ContainerErr {
    pub fn invalid_args(msg: &str) -> Self {
        Self::Args(String::from(msg))
    }
}
