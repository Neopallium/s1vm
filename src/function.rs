
use crate::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub struct InternalFunction {
  pub local_types: Vec<ValueType>,
  pub code: Vec<isa::Instruction>,
}

#[derive(Debug, Clone)]
pub struct HostFunction {
  pub mod_idx: usize,
  pub func_idx: usize,
}

#[derive(Debug)]
pub enum FunctionBody {
  Internal(InternalFunction),
  Host(HostFunction),
}

#[derive(Debug)]
pub struct Function {
  pub name: String,
  pub func_type: FunctionType,
  pub body: FunctionBody,
}

impl Function {
  pub fn new(func: &bwasm::Function) -> Result<Function> {
    let code = compile_function(func)?;
    /*
    eprintln!("----- Compiled function: {}", func.name());
    for (pc, op) in code.iter().enumerate() {
      eprintln!("- {}: {:?}", pc, op);
    }
    */
    Ok(Function {
      name: func.name().to_string(),
      func_type: FunctionType::from(func.func_type()),
      body: FunctionBody::Internal(InternalFunction{
        local_types: ValueType::from_slice(func.locals()),
        code: code,
      }),
    })
  }

  pub fn param_count(&self) -> usize {
    self.func_type.param_count()
  }

  pub fn call(&self, state: &State, store: &mut Store) -> Trap<RetValue> {
    match self.body {
      FunctionBody::Internal(ref body) => {
        // Setup stack frame for function.
        let old_bp = store.stack.push_frame(self.param_count())?;

        // run function
        let ret = run_function(self, body, state, store)?;

        // cleanup stack frame.
        store.stack.pop_frame(old_bp);
        Ok(ret)
      },
      FunctionBody::Host(_) => {
        todo!("");
      },
    }
  }
}

fn run_function(func: &Function, body: &InternalFunction, state: &State, store: &mut Store) -> Trap<RetValue> {
  // Clear stack space for locals.
  store.stack.reserve_locals(body.local_types.len())?;

  //eprintln!("run_function: {}", func.name);
  //eprintln!("-- Stack: {:?}", store.stack);
  let mut pc = 0usize;
  let code = &body.code;

  if code.len() == 0 {
    // No code!
    return Ok(None);
  }
  let pc_end = code.len() - 1;

  loop {
    use crate::isa::Instruction::*;
    let op = code[pc];
    //eprintln!("- {}: {:?}", pc, op);
    match op {
      Br(jump_pc) => {
        pc = jump_pc as usize;
        continue;
      },
      BrIfEqz(jump_pc) => {
        let val: u64 = store.stack.pop()?;
        if val == 0 {
          pc = jump_pc as usize;
          continue;
        }
      },
      BrIfNez(jump_pc) => {
        let val: u64 = store.stack.pop()?;
        if val != 0 {
          pc = jump_pc as usize;
          continue;
        }
      },

      BrTable{ .. } => {
        todo!("BrTable");
      },

      Unreachable => {
        return Err(TrapKind::Unreachable);
      },
      Return => {
        //eprintln!("-- Stack: {:?}", store.stack);
        match func.func_type.ret_type {
          Some(ret_type) => {
            return Ok(Some(store.stack.pop_typed(ret_type)?));
          },
          None => break,
        }
      },

      Call(func_idx) => {
        let ret = state.invoke_function(store, func_idx)?;
        if let Some(ret) = ret {
          //eprintln!("------------------- return: {:?}", ret);
          store.stack.push_val(ret.into())?;
        } else {
          //eprintln!("------------------- return: <no return value>");
        }
      },
      CallIndirect(_type_idx) => {
        todo!("call");
      },

      Drop => {
        todo!("drop");
      },
      Select => {
        todo!("select");
      },

      GetLocal(local_idx) => {
        let val = store.stack.get_local_val(local_idx)?;
        store.stack.push_val(val)?;
      },
      SetLocal(local_idx) => {
        let val = store.stack.pop_val()?;
        store.stack.set_local_val(local_idx, val)?;
      },
      TeeLocal(local_idx) => {
        let val = store.stack.top_val()?;
        store.stack.set_local_val(local_idx, val)?;
      },

      GetGlobal(_global_idx) => {
        todo!("global");
      },
      SetGlobal(_global_idx) => {
        todo!("global");
      },

      I32Load(offset) => i32_ops::load(store, offset)?,
      I64Load(offset) => i64_ops::load(store, offset)?,
      F32Load(offset) => f32_ops::load(store, offset)?,
      F64Load(offset) => f64_ops::load(store, offset)?,
      I32Load8S(offset) => i32_ops::load8_s(store, offset)?,
      I32Load8U(offset) => i32_ops::load8_u(store, offset)?,
      I32Load16S(offset) => i32_ops::load16_s(store, offset)?,
      I32Load16U(offset) => i32_ops::load16_u(store, offset)?,
      I64Load8S(offset) => i64_ops::load8_s(store, offset)?,
      I64Load8U(offset) => i64_ops::load8_u(store, offset)?,
      I64Load16S(offset) => i64_ops::load16_s(store, offset)?,
      I64Load16U(offset) => i64_ops::load16_u(store, offset)?,
      I64Load32S(offset) => i64_ops::load32_s(store, offset)?,
      I64Load32U(offset) => i64_ops::load32_u(store, offset)?,
      I32Store(offset) => i32_ops::store(store, offset)?,
      I64Store(offset) => i64_ops::store(store, offset)?,
      F32Store(offset) => f32_ops::store(store, offset)?,
      F64Store(offset) => f64_ops::store(store, offset)?,
      I32Store8(offset) => i32_ops::store8(store, offset)?,
      I32Store16(offset) => i32_ops::store16(store, offset)?,
      I64Store8(offset) => i64_ops::store8(store, offset)?,
      I64Store16(offset) => i64_ops::store16(store, offset)?,
      I64Store32(offset) => i64_ops::store32(store, offset)?,

      CurrentMemory => {
        todo!("memory");
      },
      GrowMemory => {
        todo!("memory");
      },

      I32Const(val) => store.stack.push(val)?,
      I64Const(val) => store.stack.push(val)?,
      F32Const(val) => store.stack.push(val)?,
      F64Const(val) => store.stack.push(val)?,

      I32Eqz => i32_ops::eqz(store)?,
      I32Eq => i32_ops::eq(store)?,
      I32Ne => i32_ops::ne(store)?,
      I32LtS => i32_ops::lt_s(store)?,
      I32LtU => i32_ops::lt_u(store)?,
      I32GtS => i32_ops::gt_s(store)?,
      I32GtU => i32_ops::gt_u(store)?,
      I32LeS => i32_ops::le_s(store)?,
      I32LeU => i32_ops::le_u(store)?,
      I32GeS => i32_ops::ge_s(store)?,
      I32GeU => i32_ops::ge_u(store)?,

      I64Eqz => i64_ops::eqz(store)?,
      I64Eq => i64_ops::eq(store)?,
      I64Ne => i64_ops::ne(store)?,
      I64LtS => i64_ops::lt_s(store)?,
      I64LtU => i64_ops::lt_u(store)?,
      I64GtS => i64_ops::gt_s(store)?,
      I64GtU => i64_ops::gt_u(store)?,
      I64LeS => i64_ops::le_s(store)?,
      I64LeU => i64_ops::le_u(store)?,
      I64GeS => i64_ops::ge_s(store)?,
      I64GeU => i64_ops::ge_u(store)?,

      F32Eq => f32_ops::eq(store)?,
      F32Ne => f32_ops::ne(store)?,
      F32Lt => f32_ops::lt(store)?,
      F32Gt => f32_ops::gt(store)?,
      F32Le => f32_ops::le(store)?,
      F32Ge => f32_ops::ge(store)?,

      F64Eq => f64_ops::eq(store)?,
      F64Ne => f64_ops::ne(store)?,
      F64Lt => f64_ops::lt(store)?,
      F64Gt => f64_ops::gt(store)?,
      F64Le => f64_ops::le(store)?,
      F64Ge => f64_ops::ge(store)?,

      I32Clz => i32_ops::clz(store)?,
      I32Ctz => i32_ops::ctz(store)?,
      I32Popcnt => i32_ops::popcnt(store)?,
      I32Add => i32_ops::add(store)?,
      I32Sub => i32_ops::sub(store)?,
      I32Mul => i32_ops::mul(store)?,
      I32DivS => i32_ops::div_s(store)?,
      I32DivU => i32_ops::div_u(store)?,
      I32RemS => i32_ops::rem_s(store)?,
      I32RemU => i32_ops::rem_u(store)?,
      I32And => i32_ops::and(store)?,
      I32Or => i32_ops::or(store)?,
      I32Xor => i32_ops::xor(store)?,
      I32Shl => i32_ops::shl(store)?,
      I32ShrS => i32_ops::shr_s(store)?,
      I32ShrU => i32_ops::shr_u(store)?,
      I32Rotl => i32_ops::rotl(store)?,
      I32Rotr => i32_ops::rotr(store)?,

      I64Clz => i64_ops::clz(store)?,
      I64Ctz => i64_ops::ctz(store)?,
      I64Popcnt => i64_ops::popcnt(store)?,
      I64Add => i64_ops::add(store)?,
      I64Sub => i64_ops::sub(store)?,
      I64Mul => i64_ops::mul(store)?,
      I64DivS => i64_ops::div_s(store)?,
      I64DivU => i64_ops::div_u(store)?,
      I64RemS => i64_ops::rem_s(store)?,
      I64RemU => i64_ops::rem_u(store)?,
      I64And => i64_ops::and(store)?,
      I64Or => i64_ops::or(store)?,
      I64Xor => i64_ops::xor(store)?,
      I64Shl => i64_ops::shl(store)?,
      I64ShrS => i64_ops::shr_s(store)?,
      I64ShrU => i64_ops::shr_u(store)?,
      I64Rotl => i64_ops::rotl(store)?,
      I64Rotr => i64_ops::rotr(store)?,

      F32Abs => f32_ops::abs(store)?,
      F32Neg => f32_ops::neg(store)?,
      F32Ceil => f32_ops::ceil(store)?,
      F32Floor => f32_ops::floor(store)?,
      F32Trunc => f32_ops::trunc(store)?,
      F32Nearest => f32_ops::nearest(store)?,
      F32Sqrt => f32_ops::sqrt(store)?,
      F32Add => f32_ops::add(store)?,
      F32Sub => f32_ops::sub(store)?,
      F32Mul => f32_ops::mul(store)?,
      F32Div => f32_ops::div(store)?,
      F32Min => f32_ops::min(store)?,
      F32Max => f32_ops::max(store)?,
      F32Copysign => f32_ops::copysign(store)?,
      F64Abs => f64_ops::abs(store)?,
      F64Neg => f64_ops::neg(store)?,
      F64Ceil => f64_ops::ceil(store)?,
      F64Floor => f64_ops::floor(store)?,
      F64Trunc => f64_ops::trunc(store)?,
      F64Nearest => f64_ops::nearest(store)?,
      F64Sqrt => f64_ops::sqrt(store)?,
      F64Add => f64_ops::add(store)?,
      F64Sub => f64_ops::sub(store)?,
      F64Mul => f64_ops::mul(store)?,
      F64Div => f64_ops::div(store)?,
      F64Min => f64_ops::min(store)?,
      F64Max => f64_ops::max(store)?,
      F64Copysign => f64_ops::copysign(store)?,

      I32WrapI64 => {
        todo!();
      },
      I32TruncSF32 => i32_ops::trunc_s_f32(store)?,
      I32TruncUF32 => i32_ops::trunc_u_f32(store)?,
      I32TruncSF64 => i32_ops::trunc_s_f64(store)?,
      I32TruncUF64 => i32_ops::trunc_u_f64(store)?,
      I64ExtendSI32 => {
        todo!();
      },
      I64ExtendUI32 => {
        todo!();
      },
      I64TruncSF32 => i64_ops::trunc_s_f32(store)?,
      I64TruncUF32 => i64_ops::trunc_u_f32(store)?,
      I64TruncSF64 => i64_ops::trunc_s_f64(store)?,
      I64TruncUF64 => i64_ops::trunc_u_f64(store)?,

      F32ConvertSI32 => f32_ops::convert_s_i32(store)?,
      F32ConvertUI32 => f32_ops::convert_u_i32(store)?,
      F32ConvertSI64 => f32_ops::convert_s_i64(store)?,
      F32ConvertUI64 => f32_ops::convert_u_i64(store)?,
      F32DemoteF64 => {
        todo!();
      },
      F64ConvertSI32 => f64_ops::convert_s_i32(store)?,
      F64ConvertUI32 => f64_ops::convert_u_i32(store)?,
      F64ConvertSI64 => f64_ops::convert_s_i64(store)?,
      F64ConvertUI64 => f64_ops::convert_u_i64(store)?,
      F64PromoteF32 => {
        todo!();
      },

      I32ReinterpretF32 => {
        todo!();
      },
      I64ReinterpretF64 => {
        todo!();
      },
      F32ReinterpretI32 => {
        todo!();
      },
      F64ReinterpretI64 => {
        todo!();
      },
    }
    if pc == pc_end {
      break;
    }
    pc = pc + 1;
  }
  Ok(None)
}

macro_rules! impl_int_binops {
  ($store: expr, $type: ty, $op: ident) => {
    {
      let (left, right) = $store.stack.pop_pair()? as ($type, $type);
      let res = left.$op(right);
      $store.stack.push(res)
    }
  };
  ($store: expr, $type: ty, $op: ident, $as_type: ty) => {
    {
      let (left, right) = $store.stack.pop_pair()? as ($type, $type);
      let res = left.$op(right);
      $store.stack.push(res as $as_type)
    }
  };
  ($store: expr, $type: ty, $type2: ty, $op: ident, $as_type: ty) => {
    {
      let left: $type = $store.stack.pop()?;
      let right: $type2 = $store.stack.pop()?;
      let res = left.$op(right);
      $store.stack.push(res as $as_type)
    }
  };
  ($store: expr, $type: ty, $op: ident, $as_type: ty, $mask: expr) => {
    {
      let (left, right) = $store.stack.pop_pair()? as ($type, $type);
      let res = left.$op((right & $mask) as u32);
      $store.stack.push(res as $as_type)
    }
  };
}

macro_rules! impl_int_binops_div {
  ($store: expr, $type: ty, $op: ident, $as_type: ty) => {
    {
      let (left, right) = $store.stack.pop_pair()? as ($type, $type);
      let res = left.$op(right)
        .ok_or_else(|| {
          if right == 0 {
            TrapKind::DivisionByZero
          } else {
            TrapKind::InvalidConversionToInt
          }
        })?;
      $store.stack.push(res as $as_type)
    }
  };
}

macro_rules! impl_int_relops {
  ($store: expr, $type: ty, $relop: expr) => {
    {
      let val: $type = $store.stack.pop()?;
      let res = $relop(val);
      $store.stack.push(res as u64)
    }
  };
  ($store: expr, $type: ty, $type2: ty, $relop: expr) => {
    {
      let right: $type2 = $store.stack.pop()?;
      let left: $type = $store.stack.pop()?;
      let res = $relop(left, right);
      $store.stack.push(res as u64)
    }
  };
}

macro_rules! impl_numeric_ops {
  ($op_mod: ident, $type: ty, $type_u: ty) => {
    #[allow(dead_code)]
    mod $op_mod {
      use std::ops::*;
      use super::*;

      pub fn load(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load8_s(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load8_u(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load16_s(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load16_u(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load32_s(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load32_u(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn store(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn store8(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn store16(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn store32(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn clz(store: &mut Store) -> Trap<()> {
        let val: $type = store.stack.pop()?;
        store.stack.push(val.leading_zeros())
      }
      pub fn ctz(store: &mut Store) -> Trap<()> {
        let val: $type = store.stack.pop()?;
        store.stack.push(val.trailing_zeros())
      }
      pub fn popcnt(store: &mut Store) -> Trap<()> {
        let val: $type = store.stack.pop()?;
        store.stack.push(val.count_ones())
      }

      pub fn add(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, wrapping_add)
      }

      pub fn sub(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, wrapping_sub)
      }

      pub fn mul(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, wrapping_mul)
      }
      pub fn div_s(store: &mut Store) -> Trap<()> {
        impl_int_binops_div!(store, $type, checked_div, i64)
      }
      pub fn div_u(store: &mut Store) -> Trap<()> {
        impl_int_binops_div!(store, $type, checked_div, u64)
      }
      pub fn rem_s(store: &mut Store) -> Trap<()> {
        impl_int_binops_div!(store, $type, checked_rem, i64)
      }
      pub fn rem_u(store: &mut Store) -> Trap<()> {
        impl_int_binops_div!(store, $type, checked_rem, u64)
      }
      pub fn and(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, bitand)
      }
      pub fn or(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, bitor)
      }
      pub fn xor(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, bitxor)
      }
      pub fn shl(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, wrapping_shl, $type_u, 0x1F)
      }
      pub fn shr_s(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, wrapping_shr, $type_u, 0x1F)
      }
      pub fn shr_u(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, wrapping_shr, $type_u, 0x1F)
      }
      pub fn rotl(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, u32, rotate_left, u64)
      }
      pub fn rotr(store: &mut Store) -> Trap<()> {
        impl_int_binops!(store, $type, u32, rotate_right, u64)
      }

      pub fn eqz(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, |val| {
          val == Default::default()
        })
      }
      pub fn eq(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, $type, |left, right| {
          left == right
        })
      }
      pub fn ne(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, $type, |left, right| {
          left != right
        })
      }
      pub fn lt_s(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, $type, |left, right| {
          left < right
        })
      }
      pub fn lt_u(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type_u, $type_u, |left, right| {
          left < right
        })
      }
      pub fn gt_s(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, $type, |left, right| {
          left > right
        })
      }
      pub fn gt_u(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type_u, $type_u, |left, right| {
          left > right
        })
      }
      pub fn le_s(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, $type, |left, right| {
          left <= right
        })
      }
      pub fn le_u(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type_u, $type_u, |left, right| {
          left <= right
        })
      }
      pub fn ge_s(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type, $type, |left, right| {
          left >= right
        })
      }
      pub fn ge_u(store: &mut Store) -> Trap<()> {
        impl_int_relops!(store, $type_u, $type_u, |left, right| {
          left >= right
        })
      }

      pub fn trunc_s_f32(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn trunc_u_f32(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn trunc_s_f64(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn trunc_u_f64(_store: &mut Store) -> Trap<()> {
        todo!();
      }

    }
  };
}

impl_numeric_ops!(i32_ops, i32, u32);
impl_numeric_ops!(i64_ops, i64, u64);

macro_rules! impl_float_numeric_ops {
  ($op_mod: ident, $type: ty) => {
    #[allow(dead_code)]
    mod $op_mod {

      use super::*;

      pub fn load(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn store(_store: &mut Store, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn abs(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn neg(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn ceil(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn floor(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn trunc(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn nearest(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn sqrt(_store: &mut Store) -> Trap<()> {
        todo!();
      }

      pub fn add(store: &mut Store) -> Trap<()> {
        let (left, right) = store.stack.pop_pair()? as ($type, $type);
        let res = left + right;
        store.stack.push(res)?;
        Ok(())
      }

      pub fn sub(store: &mut Store) -> Trap<()> {
        let (left, right) = store.stack.pop_pair()? as ($type, $type);
        let res = left - right;
        store.stack.push(res)?;
        Ok(())
      }

      pub fn mul(store: &mut Store) -> Trap<()> {
        let (left, right) = store.stack.pop_pair()? as ($type, $type);
        let res = left * right;
        store.stack.push(res)?;
        Ok(())
      }
      pub fn div(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn min(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn max(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn copysign(_store: &mut Store) -> Trap<()> {
        todo!();
      }

      pub fn eq(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn ne(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn lt(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn gt(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn le(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn ge(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn convert_s_i32(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn convert_u_i32(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn convert_s_i64(_store: &mut Store) -> Trap<()> {
        todo!();
      }
      pub fn convert_u_i64(_store: &mut Store) -> Trap<()> {
        todo!();
      }
    }
  };
}

impl_float_numeric_ops!(f32_ops, f32);
impl_float_numeric_ops!(f64_ops, f64);

