use std::collections::HashMap;

use crate::*;
use crate::error::*;

/// VM Store - Mutable data
#[derive(Default)]
pub struct Store {
  pub mem: Vec<u8>,
  pub stack: Stack,
}

/// VM State - Immutable, only changes when loading a module.
#[derive(Default)]
pub struct State {
  funcs: Vec<Function>,
  // Loaded modules
  module_instances: Vec<ModuleInstance>,
  modules: HashMap<String, ModuleInstanceAddr>,
}

impl State {
  pub fn new() -> State {
    State {
      ..Default::default()
    }
  }

  pub fn load_file(&mut self, name: &str, file: &str) -> Result<ModuleInstanceAddr> {
    if self.modules.contains_key(name) {
      return Err(Error::ModuleExists)
    }
    // load new module from file.
    let module = bwasm::Module::from_file(file)?;

    self.compile_module(name, &module)
  }

  fn compile_module(&mut self, name: &str, module: &bwasm::Module) -> Result<ModuleInstanceAddr> {
    let mod_addr = self.module_instances.len() as ModuleInstanceAddr;
    // create new module instance.
    let mut mod_inst = ModuleInstance::new();
    // load functions
    for func in module.functions().into_iter() {
      let addr = self.funcs.len() as FuncAddr;
      self.funcs.push(Function::new(&func)?);
      mod_inst.add_function(addr);
    }
    // load exports
    for export in module.exports().into_iter() {
      match export.internal() {
        bwasm::Internal::Function(idx) => {
          //eprintln!("-Export function '{}' at {}", export.field(), *idx);
          mod_inst.add_export(export.field(), *idx)?;
        },
        _ => {
          println!("Unhandled export: {:?}", export);
        },
      }
    }
    // Finished compile module.
    self.module_instances.push(mod_inst);
    self.modules.insert(name.to_string(), mod_addr);
    Ok(mod_addr)
  }

  pub fn get_function(&self, addr: FuncAddr) -> Trap<&Function> {
    self.funcs.get(addr as usize).ok_or(TrapKind::InvalidFunctionAddr)
  }

  fn get_module_instance(&self, module: &str) -> Result<&ModuleInstance> {
    if let Some(addr) = self.modules.get(module) {
      if let Some(inst) = self.module_instances.get(*addr as usize) {
        return Ok(inst)
      }
    }
    Err(Error::ModuleNotFound)
  }

  pub fn get_exported(&self, module: &str, name: &str) -> Result<FuncAddr> {
    let mod_inst = self.get_module_instance(module)?;
    mod_inst.find_function(name)
  }

  pub fn invoke_function(&self, store: &mut Store, func_addr: FuncAddr) -> Trap<()> {
    let func = self.get_function(func_addr)?;
    func.call(self, store)
  }

  pub fn call(&self, store: &mut Store, func_addr: FuncAddr, params: &[Value]) -> Result<RetValue> {
    store.stack.push_params(params)?;
    let func = self.get_function(func_addr)?;
    func.call(self, store)?;
    if let Some(ret_type) = func.ret_type() {
      let ret = store.stack.pop_typed(ret_type)?;
      Ok(Some(ret))
    } else {
      Ok(None)
    }
  }
}

#[derive(Default)]
pub struct VM {
  // Mutable store
  store: Store,

  /// Immutable state.  Only mutable when loading modules.
  state: State,
}

impl VM {
  pub fn new() -> VM {
    VM {
      ..Default::default()
    }
  }

  pub fn load_file(&mut self, name: &str, file: &str) -> Result<ModuleInstanceAddr> {
    self.state.load_file(name, file)
  }

  pub fn get_exported(&self, module: &str, name: &str) -> Result<FuncAddr> {
    self.state.get_exported(module, name)
  }

  pub fn call(&mut self, module: &str, name: &str, params: &[Value]) -> Result<RetValue> {
    let func_addr = self.state.get_exported(module, name)?;
    self.state.call(&mut self.store, func_addr, params)
  }
}

