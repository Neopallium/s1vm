use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapKind {
  InvalidFunctionAddr,

  Unreachable,
  MemoryAccessOutOfBounds,
  TableAccessOutOfBounds,
  ElemUninitialized,
  DivisionByZero,
  InvalidConversionToInt,
  StackOverflow,
  UnexpectedSignature,
}
pub type Trap<T, K = TrapKind> = std::result::Result<T, K>;

#[derive(Debug, Clone)]
pub enum Error {
  FuncNotFound,
  FuncExists,

  ModuleNotFound,
  ModuleExists,

  ParseError(parity_wasm::SerializationError),
  ValidationError(String),

  RuntimeError(TrapKind),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::FuncNotFound => write!(f, "function not found"),
      Error::FuncExists => write!(f, "function already exists"),
      Error::ModuleNotFound => write!(f, "module not found"),
      Error::ModuleExists => write!(f, "module already exists"),
      Error::ParseError(e) => write!(f, "failed to parse wasm: {}", e),
      Error::ValidationError(e) => write!(f, "failed to validate wasm: {}", e),
      Error::RuntimeError(trap) => write!(f, "runtime trap: {:?}", trap),
    }
  }
}

impl std::error::Error for Error {
}

impl From<TrapKind> for Error {
  fn from(trap: TrapKind) -> Self {
    Error::RuntimeError(trap)
  }
}

impl From<parity_wasm::SerializationError> for Error {
  fn from(error: parity_wasm::SerializationError) -> Self {
    Error::ParseError(error)
  }
}

impl From<bwasm::LoadError> for Error {
  fn from(error: bwasm::LoadError) -> Self {
    match error {
      bwasm::LoadError::SerializationError(error) => Error::ParseError(error),
      bwasm::LoadError::ValidationError(error) => Error::ValidationError(format!("{}", error)),
    }
  }
}

