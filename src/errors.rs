use std::io;

#[derive(Debug)]
pub enum SernetError {
    IoError(io::Error),
    OtherError,
}

impl From<io::Error> for SernetError {
    fn from(error: io::Error) -> Self {
        SernetError::IoError(error)
    }
}
