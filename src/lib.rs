#![forbid(unsafe_code)]

pub mod error;
pub use error::Error;

// VM
mod stack;
mod memory;
mod export;
mod value;
pub use value::*;
mod op;
pub use op::*;
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
