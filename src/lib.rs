#![forbid(unsafe_code)]

pub mod error;
pub use error::{Error, Result};

// VM
mod memory;
mod export;
mod stack;
pub use stack::*;
mod value;
pub use value::*;
mod isa;
pub use isa::*;
mod vm;
pub use vm::*;

// Module
mod function;
pub use function::*;
mod module;
pub use module::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
