// TODO: error messages (custom Debug impl)

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    UnmatchedLoopStart { lineno: usize, column: usize },
    UnmatchedLoopEnd { lineno: usize, column: usize },
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

impl Error {
    pub fn print(&self, out: &mut std::io::Write, input: &str) -> std::io::Result<()> {
        match *self {
            Error::IO(ref err) => write!(out, "error:{}: {}\n", input, err),

            Error::UnmatchedLoopStart { lineno, column } =>
                write!(out, "error:{}:{}:{}: unmatched '['\n", input, lineno, column),

            Error::UnmatchedLoopEnd { lineno, column } =>
                write!(out, "error:{}:{}:{}: unmatched ']'\n", input, lineno, column),
        }
    }
}