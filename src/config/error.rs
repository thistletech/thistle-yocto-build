use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Parse error")]
    Parse(#[from] serde_yaml::Error),
}
