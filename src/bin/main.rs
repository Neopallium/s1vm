#![forbid(unsafe_code)]

use s1vm::*;

fn main() -> Result<(), Error> {
  let mut args = std::env::args();
  args.next(); // skip program name.
  let file = args.next().expect("missing file name");
  let func = args.next().expect("missing function name");
  let params: Vec<Value> = args.map(|x| {
    match x.parse::<i64>() {
      Ok(v) => Value::I64(v),
      Err(e) => {
        eprintln!("failed to parse '{}': {}", x, e);
        Value::I64(0)
      },
    }
  }).collect();

  //println!("Type sizes:");
  //println!("isa::Instruction = {}", std::mem::size_of::<Instruction>());
  //println!("bwasm::Instruction = {}", std::mem::size_of::<bwasm::Instruction>());

  // Create VM.
  let mut vm = VM::new();

  // Load wasm file
  //println!("--- Loading module: {}", file);
  vm.load_file("main", &file)?;

  let mut instance = vm.spawn();
  // Call module function
  //println!("Calling:  {}({:?})", func, params);
  let ret = instance.call("main", &func, &params)?;
  if let Some(ret) = ret {
    println!("{}", ret);
  } else {
    println!("ret = <no return value>");
  }

  Ok(())
}

