use std::collections::HashMap;
use std::sync::Arc;

use crate::*;
use crate::compiler::Compiler;
use crate::error::*;

/// VM Store - Mutable data
pub struct Store {
  /// Immutable state.  Only mutable when loading modules.
  state: Arc<State>,

  pub mem: Vec<u8>,
  pub stack: Stack,
}

impl Store {
  pub fn new(state: Arc<State>) -> Self {
    Self {
      state: state.clone(),
      mem: Default::default(),
      stack: Default::default(),
    }
  }

  pub fn get_exported(&self, module: &str, name: &str) -> Result<FuncAddr> {
    self.state.get_exported(module, name)
  }

  pub fn invoke_function(&mut self, func_addr: FuncAddr, l0: &mut StackValue) -> Trap<Option<StackValue>> {
    let state = self.state.clone();
    let func = state.get_function(func_addr)?;
    func.call(self, l0)
  }

  pub fn call(&mut self, func_addr: FuncAddr, params: &[Value]) -> Result<RetValue> {
    let state = self.state.clone();
    self.stack.push_params(params)?;
    let func = state.get_function(func_addr)?;
    let mut l0 = StackValue::from(params[0]);
    let ret = func.call(self, &mut l0)?;
    if let Some(ret) = ret {
      if let Some(ret_type) = func.ret_type() {
        Ok(Some(match ret_type {
          ValueType::I32 => Value::I32(ret.0 as _),
          ValueType::I64 => Value::I64(ret.0 as _),
          ValueType::F32 => Value::F32(f32::from_bits(ret.0 as _)),
          ValueType::F64 => Value::F64(f64::from_bits(ret.0 as _)),
        }))
      } else {
        Err(Error::RuntimeError(TrapKind::UnexpectedSignature))
      }
    } else {
      Ok(None)
    }
  }
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
    // compile functions
    let compiler = Compiler::new(module);
    self.funcs = compiler.compile()?;
    for addr in 0..self.funcs.len() {
      mod_inst.add_function(addr as u32);
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

  /*
  pub fn invoke_function(&self, store: &mut Store, func_addr: FuncAddr, l0: &mut StackValue) -> Trap<Option<StackValue>> {
    let func = self.get_function(func_addr)?;
    func.call(self, store, l0)
  }

  pub fn call(&self, store: &mut Store, func_addr: FuncAddr, params: &[Value]) -> Result<RetValue> {
    store.stack.push_params(params)?;
    let func = self.get_function(func_addr)?;
    let mut l0 = StackValue::from(params[0]);
    let ret = func.call(self, store, &mut l0)?;
    if let Some(ret) = ret {
      if let Some(ret_type) = func.ret_type() {
        Ok(Some(match ret_type {
          ValueType::I32 => Value::I32(ret.0 as _),
          ValueType::I64 => Value::I64(ret.0 as _),
          ValueType::F32 => Value::F32(f32::from_bits(ret.0 as _)),
          ValueType::F64 => Value::F64(f64::from_bits(ret.0 as _)),
        }))
      } else {
        Err(Error::RuntimeError(TrapKind::UnexpectedSignature))
      }
    } else {
      Ok(None)
    }
  }
  */
}

pub struct VMInstance {
  // Mutable store
  store: Store,
}

impl VMInstance {
  fn new(state: Arc<State>) -> Self {
    Self {
      store: Store::new(state),
    }
  }

  pub fn call(&mut self, module: &str, name: &str, params: &[Value]) -> Result<RetValue> {
    let func_addr = self.store.get_exported(module, name)?;
    self.store.call(func_addr, params)
  }
}

pub struct VM {
  // Immutable state.  Only mutable when loading modules.
  state: Arc<State>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      state: Arc::new(State::default()),
    }
  }

  pub fn load_file(&mut self, name: &str, file: &str) -> Result<ModuleInstanceAddr> {
    if let Some(state) = Arc::get_mut(&mut self.state) {
      state.load_file(name, file)
    } else {
      // TODO: Better error message.
      Err(Error::CannotModifySharedVM)
    }
  }

  pub fn get_exported(&self, module: &str, name: &str) -> Result<FuncAddr> {
    self.state.get_exported(module, name)
  }

  pub fn spawn(&self) -> VMInstance {
    VMInstance::new(self.state.clone())
  }
}

