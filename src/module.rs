use std::collections::HashMap;

use crate::*;
use crate::error::*;

pub struct ModuleInstance<'a> {
  module: &'a Module,
}

impl<'a> ModuleInstance<'a> {
  pub fn new(module: &'a Module) -> ModuleInstance {
    ModuleInstance {
      module,
    }
  }

  pub fn name(&self) -> &str {
    self.module.name()
  }

  pub fn call(&mut self, name: &str, params: &[Value]) -> Result<Value> {
    let func = self.module.get_function(name)?;
    func.call(self, params)
  }
}

#[derive(Default)]
pub struct Module {
  name: String,
  //types: Vec<FuncType>,
  funcs: Vec<Function>,
  //globals: Vec<Value>,
  exports: HashMap<String, FuncIdx>,
}

impl Module {
  pub fn new(name: &str) -> Module {
    Module{
      name: name.to_string(),
      ..Default::default()
    }
  }

  pub fn name(&self) -> &str {
    self.name.as_str()
  }

  // Add and export a function
  pub fn add_function(&mut self, name: &str, function: Function) -> Result<FuncIdx> {
    let name = name.to_string();
    if self.exports.contains_key(&name) {
      Err(Error::FuncExists)
    } else {
      let idx = self.funcs.len() as FuncIdx;
      self.funcs.push(function);
      self.exports.insert(name, idx);
      Ok(idx)
    }
  }

  pub fn get_function(&self, name: &str) -> Result<&Function> {
    if let Some(idx) = self.exports.get(&name.to_string()) {
      if let Some(func) = self.funcs.get(*idx as usize) {
        return Ok(func);
      }
    }
    Err(Error::FuncNotFound)
  }
}

