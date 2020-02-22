use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
  FuncNotFound,
  FuncExists,

  ModuleNotFound,
  ModuleExists,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Error::FuncNotFound => write!(f, "function not found"),
      Error::FuncExists => write!(f, "function already exists"),
      Error::ModuleNotFound => write!(f, "module not found"),
      Error::ModuleExists => write!(f, "module already exists"),
    }
  }
}

impl std::error::Error for Error {
}

