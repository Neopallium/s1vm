use std::fmt;

pub enum ValType {
  I32,
  I64,
  F32,
  F64,
}

#[derive(Debug)]
pub enum Value {
  I32(i32),
  I64(i64),
  F32(i32),
  F64(i64),
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

pub struct FuncType {
  pub params: Vec<ValType>,
  pub ret_type: ValType,
}

impl FuncType {
  pub fn new(params: Vec<ValType>, ret_type: ValType) -> FuncType {
    FuncType {
      params,
      ret_type,
    }
  }
}

impl Default for FuncType {
  fn default() -> Self {
    Self::new(vec!(), ValType::I64)
  }
}

