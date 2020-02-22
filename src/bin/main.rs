#![forbid(unsafe_code)]

use s1vm::*;

fn main() -> Result<(), Error> {
  // Compile module
  let func = Function::new("add");
  let mut module = Module::new("test");
  module.add_function("add", func)?;

  // Create VM and load module into it.
  let mut vm = VM::new();
  let _mod_addr = vm.load_module(&module)?;

  // Call module function
  let ret = vm.call("test", "add", &vec![Value::I64(1),Value::I64(2)])?;
  println!("add(1, 2) = {}", ret);

  Ok(())
}

