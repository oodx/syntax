use std::fmt;

#[derive(Debug)]
pub enum SyntaxError {
    InvalidArgument(String),
    RenderError(String),
    ResolveError(String),
    ExecError(String),
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::InvalidArgument(s) => write!(f, "Invalid argument: {}", s),
            SyntaxError::RenderError(s) => write!(f, "Render error: {}", s),
            SyntaxError::ResolveError(s) => write!(f, "Resolve error: {}", s),
            SyntaxError::ExecError(s) => write!(f, "Exec error: {}", s),
        }
    }
}

impl std::error::Error for SyntaxError {}

