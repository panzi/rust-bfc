// TODO: error messages (custom Debug impl)

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    UnmatchedLoopStart(usize, usize, usize, usize),
    UnmatchedLoopEnd(usize, usize),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}
