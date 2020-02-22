
use crate::value::*;

pub enum OpCode {
  // variable instructions
  GetLocal(LocalIdx),
  SetLocal(LocalIdx),
  TeeLocal(LocalIdx),
  GetGlobal(GlobalIdx),
  SetGlobal(GlobalIdx),
  TeeGlobal(GlobalIdx),

  // ibinop
  Add,
  Sub,
  Mul,
  And,
  Or,
  Xor,

  // Control flow
  Jump(u32),
  Call(FuncIdx),
  Ret,
}

