use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
  I32,
  I64,
  F32,
  F64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::I32(v) => write!(f, "{}", v),
      Value::I64(v) => write!(f, "{}", v),
      Value::F32(v) => write!(f, "{}", v),
      Value::F64(v) => write!(f, "{}", v),
    }
  }
}

impl From<bwasm::ValueType> for ValueType {
  fn from(val_type: bwasm::ValueType) -> Self {
    match val_type {
      bwasm::ValueType::I32 => ValueType::I32,
      bwasm::ValueType::I64 => ValueType::I64,
      bwasm::ValueType::F32 => ValueType::F32,
      bwasm::ValueType::F64 => ValueType::F64,
    }
  }
}

impl From<&bwasm::ValueType> for ValueType {
  fn from(val_type: &bwasm::ValueType) -> Self {
    ValueType::from(*val_type)
  }
}

impl ValueType {
  pub fn from_slice(val_types: &[bwasm::ValueType]) -> Vec<ValueType> {
    val_types.iter().map(ValueType::from).collect()
  }
}

pub type RetValue = Option<Value>;

pub type ConstI32 = i32;
pub type ConstI64 = i64;
pub type ConstF32 = f32;
pub type ConstF64 = f64;

pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemIdx = u32;
pub type GlobalIdx = u32;
pub type LocalIdx = u32;
pub type LabelIdx = u32;

pub type ModuleInstanceAddr = u32;
pub type FuncAddr = u32;
pub type TableAddr = u32;
pub type MemAddr = u32;
pub type GlobalAddr = u32;

#[derive(Debug, Clone, Default)]
pub struct FunctionType {
  pub params: Vec<ValueType>,
  pub ret_type: Option<ValueType>,
}

impl FunctionType {
  pub fn new() -> FunctionType {
    Default::default()
  }

  pub fn param_count(&self) -> usize {
    self.params.len()
  }
}

impl From<bwasm::FunctionType> for FunctionType {
  fn from(func_type: bwasm::FunctionType) -> Self {
    FunctionType::from(&func_type)
  }
}

impl From<&bwasm::FunctionType> for FunctionType {
  fn from(func_type: &bwasm::FunctionType) -> Self {
    FunctionType {
      params: ValueType::from_slice(func_type.params()),
      ret_type: func_type.return_type().map(ValueType::from),
    }
  }
}

