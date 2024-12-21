#[derive(Debug)]
pub enum ContainerErr {
    Args(String),
    Bundle(String),
    Child(String),
}

impl ContainerErr {
    pub fn invalid_args(msg: &str) -> Self {
        Self::Args(String::from(msg))
    }
}
