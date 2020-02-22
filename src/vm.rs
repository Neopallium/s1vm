use std::collections::HashMap;

use crate::*;
use crate::error::*;

#[derive(Default)]
pub struct VM<'a> {
  // Store
  //funcs: Vec<FuncInstance<'a>>,

  // Loaded modules
  module_instances: Vec<ModuleInstance<'a>>,
  modules: HashMap<&'a str, ModuleInstanceAddr>,
}

impl<'a> VM<'a> {
  pub fn new() -> VM<'a> {
    VM {
      ..Default::default()
    }
  }

  pub fn load_module(&mut self, module: &'a Module) -> Result<ModuleInstanceAddr> {
    let name = module.name();
    if self.modules.contains_key(name) {
      Err(Error::ModuleExists)
    } else {
      let inst = ModuleInstance::new(module);
      let inst_addr = self.module_instances.len() as ModuleInstanceAddr;
      self.module_instances.push(inst);
      self.modules.insert(name, inst_addr);
      Ok(inst_addr)
    }
  }

  fn get_module_instance(&mut self, module: &str) -> Result<&mut ModuleInstance<'a>> {
    if let Some(addr) = self.modules.get(module) {
      if let Some(inst) = self.module_instances.get_mut(*addr as usize) {
        return Ok(inst)
      }
    }
    Err(Error::ModuleNotFound)
  }

  pub fn call(&mut self, module: &str, name: &str, params: &[Value]) -> Result<Value> {
    let inst = self.get_module_instance(module)?;
     inst.call(name, params)
  }
}

