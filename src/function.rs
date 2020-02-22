
use crate::*;
use crate::error::*;

#[derive(Default)]
pub struct Function {
  name: String,
  //func_type: FuncType,
  ops: Vec<OpCode>,
}

impl Function {
  pub fn new(name: &str) -> Function {
    Function{
      name: name.to_string(),
      ..Default::default()
    }
  }

  pub fn name(&self) -> &str {
    &self.name.as_str()
  }

  pub fn push_op(&mut self, op: OpCode) {
    self.ops.push(op);
  }

  pub fn call(&self, _inst: &mut ModuleInstance, params: &[Value]) -> Result<Value> {
    println!("Call {}({:?})", self.name, params);
    // FAKE: add(a, b) { return a + b; }
    if let Value::I64(val0) = params[0] {
      if let Value::I64(val1) = params[1] {
        return Ok(Value::I64(val0 + val1));
      }
    }
    Ok(Value::I64(0))
  }
}
