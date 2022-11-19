use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum KvError {
    #[error("Cannot process command {0} with table: {1}, key: {2}. Error: {3}")]
    StorageError(&'static str, String, String, String),
}
