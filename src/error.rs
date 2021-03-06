use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("incorect packet id: expected = {0}, recieved = {1}")]
    IncorectPacketId(i32, i32),

    #[error("connection unexpectedly closed: {:?}", .0.kind())]
    UnexpectedDisconect(io::Error),

    #[error("unknown io error: {0}")]
    OtherIo(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        // need to look into causes of commented kinds
        match err.kind() {
            // io::ErrorKind::ConnectionReset |
            io::ErrorKind::ConnectionAborted |
            // io::ErrorKind::BrokenPipe |
            io::ErrorKind::TimedOut |
            // io::ErrorKind::WriteZero |
            io::ErrorKind::UnexpectedEof => Self::UnexpectedDisconect(err),

            _ => Self::OtherIo(err),
        }
    }
}
