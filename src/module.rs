use std::collections::HashMap;

use crate::*;

#[derive(Debug, Default)]
pub struct ModuleInstance {
  //types: Vec<FuncType>,
  funcs: Vec<FuncAddr>,
  //globals: Vec<Value>,
  exports: HashMap<String, FuncIdx>,
}

impl ModuleInstance {
  pub fn new() -> ModuleInstance {
    Default::default()
  }

  // Map function idx to address
  pub fn add_function(&mut self, addr: FuncAddr) {
    self.funcs.push(addr);
  }

  // Export a function
  pub fn add_export(&mut self, name: &str, idx: FuncIdx) -> Result<()> {
    let name = name.to_string();
    if self.exports.contains_key(&name) {
      Err(Error::FuncExists)
    } else {
      self.exports.insert(name, idx);
      Ok(())
    }
  }

  pub fn find_function(&self, name: &str) -> Result<FuncAddr> {
    if let Some(idx) = self.exports.get(&name.to_string()) {
      if let Some(func) = self.funcs.get(*idx as usize) {
        return Ok(*func);
      }
    }
    Err(Error::FuncNotFound)
  }
}

