
use crate::error::*;
use crate::value::*;

pub const DEFAULT_STACK_LIMIT: usize = 1024 * 1024;
pub const INIT_STACK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct StackPtr(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct StackValue(pub u64);

#[derive(Debug, Default, Clone, Copy)]
pub struct Frame {
  /// Base Pointer - for params/locals.
  pub bp: usize,
  /// Stack Base Pointer - Base for push/pop, to make sure the function doesn't pop locals
  pub sbp: usize,
}

#[derive(Debug, Clone)]
pub struct Stack {
  stack: Vec<StackValue>,
  /// Current Frame
  frame: Frame,
  /// Maximum stack size.
  limit: usize,
}

impl Stack {
  pub fn new() -> Stack {
    Self::new_with_limit(DEFAULT_STACK_LIMIT)
  }

  pub fn new_with_limit(limit: usize) -> Stack {
    Stack {
      stack: Vec::with_capacity(INIT_STACK_SIZE),
      frame: Default::default(),
      limit,
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.stack.len()
  }

  /// How many values are on the current frame
  pub fn frame_size(&self) -> usize {
    self.len() - self.frame.sbp
  }

  fn check_overflow(&mut self, need: usize) -> Trap<usize> {
    let len = self.len();
    let space = self.limit - len;
    if space < need {
      eprintln!("StackOverflow: limit={}, len={}", self.limit, len);
      return Err(TrapKind::StackOverflow);
    }

    // Return the current stack size.
    Ok(len)
  }

  /// Start a new stack frame by saving the current base pointer.
  pub fn push_frame(&mut self, params: usize, locals: usize) -> Trap<Frame> {
    // Check if there are enough values on the current stack frame for
    // the new function's parameters.
    let cur_size = self.frame_size();
    if cur_size < params {
      eprintln!("StackOverflow: frame size={}, needed params={}", cur_size, params);
      return Err(TrapKind::StackOverflow);
    }

    // save old frame
    let old_frame = self.frame;

    // Include the params in the new stack frame.
    let bp = self.len() - params;
    self.frame = Frame{
      bp,
      sbp: bp + locals,
    };

    if locals > 0 {
      self.reserve_locals(locals);
    }

    Ok(old_frame)
  }

  pub fn reserve_locals(&mut self, locals: usize) {
    // Push initial value for locals.
    // TODO: Try improving initialization of locals.
    for _idx in 0..locals {
      //eprintln!("Push local({})", _idx);
      self.stack.push(StackValue(0));
    }
  }

  /// Remove current stack frame and restore previous frame.
  pub fn pop_frame(&mut self, old_frame: Frame) {
    // Drop current stack frame values.
    self.stack.truncate(self.frame.bp);
    // Restore old frame
    self.frame = old_frame;
  }

  pub fn push_params(&mut self, params: &[Value]) -> Trap<usize> {
    // Check for stackoverflow and get current stack size.
    let len = self.check_overflow(params.len())?;

    for val in params.iter() {
      self.stack.push(StackValue::from(*val));
    }
    // return original stack size.
    Ok(len)
  }

  pub fn drop_values(&mut self, count: u32) -> Trap<()> {
    let len = self.len();
    let new_len = len
      .checked_sub(count as usize)
      .ok_or(TrapKind::StackOverflow)?;
    self.stack.truncate(new_len);
    Ok(())
  }

  #[inline]
  pub fn tee_local(&mut self, local: LocalIdx) -> Trap<()> {
    // Copy value from stack
    let val = self.top_val()?;

    // save to local
    let idx = self.frame.bp + local as usize;
    self.stack[idx] = val;
    Ok(())
  }

  #[inline]
  pub fn set_local(&mut self, local: LocalIdx) -> Trap<()> {
    // pop value from stack
    let val = self.pop_val()?;

    // save to local
    let idx = self.frame.bp + local as usize;
    self.stack[idx] = val;
    Ok(())
  }

  #[inline]
  pub fn get_local(&mut self, local: LocalIdx) -> Trap<()> {
    let idx = self.frame.bp + local as usize;

    self.push_val(self.stack[idx])
  }

  #[inline]
  pub fn set_local_val(&mut self, local: LocalIdx, val: StackValue, l0: &mut StackValue) {
    if local == 0 {
      *l0 = val;
      return;
    }
    let idx = self.frame.bp + local as usize;

    self.stack[idx] = val;
  }

  #[inline]
  pub fn get_local_val(&mut self, local: LocalIdx, l0: &mut StackValue) -> StackValue {
    if local == 0 {
      return *l0;
    }
    let idx = self.frame.bp + local as usize;

    self.stack[idx]
  }

  #[inline]
  pub fn push_val(&mut self, val: StackValue) -> Trap<()> {
    if self.len() >=  self.limit {
      eprintln!("StackOverflow: limit={}, len={}", self.limit, self.len());
      return Err(TrapKind::StackOverflow);
    }
    //eprintln!("-- Push: {:?}", val);
    self.stack.push(val);
    Ok(())
  }

  pub fn pop_typed(&mut self, val_type: ValueType) -> Trap<Value> {
    match val_type {
      ValueType::I32 => self.pop().map(Value::I32),
      ValueType::I64 => self.pop().map(Value::I64),
      ValueType::F32 => self.pop().map(Value::F32),
      ValueType::F64 => self.pop().map(Value::F64),
    }
  }

  #[inline]
  pub fn pop_val(&mut self) -> Trap<StackValue> {
    self.stack.pop().ok_or(TrapKind::StackOverflow)
  }

  #[inline]
  pub fn top_val(&mut self) -> Trap<StackValue> {
    self.stack.last().map(|x| *x)
      .ok_or(TrapKind::StackOverflow)
  }

  /// Apply a 'unop' to top value, replacing it with the results.
  #[inline]
  pub fn unop<F>(&mut self, op: F) -> Trap<()>
    where F: FnOnce(&mut StackValue) -> Trap<()>
  {
    let mut val = self.stack.last_mut()
      .ok_or(TrapKind::StackOverflow)?;
    op(&mut val)
  }

  /// Apply a `binop` to the top two values, replacing them with the results.
  #[inline]
  pub fn binop<F>(&mut self, op: F) -> Trap<()>
    where F: FnOnce(&mut StackValue, StackValue) -> Trap<()>
  {
    let right = self.pop_val()?;
    let mut left = self.stack.last_mut()
      .ok_or(TrapKind::StackOverflow)?;
    op(&mut left, right)
  }
}

impl Default for Stack {
  fn default() -> Stack {
    Self::new()
  }
}

pub trait FromValue
where
    Self: Sized,
{
  fn from_value(val: StackValue) -> Self;
}

pub trait FromStack<T>: Sized {
  fn push(&mut self, val: T) -> Trap<()>;
  fn pop(&mut self) -> Trap<T>;

  fn pop_pair(&mut self) -> Trap<(T, T)>;
}

macro_rules! impl_stack_value {
  ($($t:ty),*) => {
    $(
      impl FromValue for $t {
        fn from_value(StackValue(val): StackValue) -> Self {
          val as _
        }
      }

      impl From<$t> for StackValue {
        fn from(other: $t) -> StackValue {
          StackValue(other as _)
        }
      }

      impl FromStack<$t> for Stack {
        #[inline]
        fn push(&mut self, val: $t) -> Trap<()> {
          self.push_val(StackValue(val as _))
        }

        #[inline]
        fn pop(&mut self) -> Trap<$t> {
          self.pop_val().map(|x| x.0 as _)
        }

        fn pop_pair(&mut self) -> Trap<($t, $t)> {
          let right = self.pop()?;
          let left = self.pop()?;
          Ok((left, right))
        }
      }
    )*
  };
}

impl_stack_value!(i8, u8, i16, u16, i32, u32, i64, u64);

macro_rules! impl_stack_value_float {
  ($($t:ty),*) => {
    $(
      impl FromValue for $t {
        fn from_value(StackValue(val): StackValue) -> Self {
          <$t>::from_bits(val as _)
        }
      }

      impl From<$t> for StackValue {
        fn from(other: $t) -> Self {
          StackValue(other.to_bits() as _)
        }
      }

      impl FromStack<$t> for Stack {
        #[inline]
        fn push(&mut self, val: $t) -> Trap<()> {
          self.push_val(StackValue(val.to_bits() as _))
        }

        #[inline]
        fn pop(&mut self) -> Trap<$t> {
          self.pop_val().map(|x| <$t>::from_bits(x.0 as _))
        }

        fn pop_pair(&mut self) -> Trap<($t, $t)> {
          let right = self.pop()?;
          let left = self.pop()?;
          Ok((left, right))
        }
      }
    )*
  };
}

impl_stack_value_float!(f32, f64);

impl From<Value> for StackValue {
  fn from(val: Value) -> StackValue {
    match val {
      Value::I32(v) => StackValue(v as _),
      Value::I64(v) => StackValue(v as _),
      Value::F32(v) => StackValue(v.to_bits() as _),
      Value::F64(v) => StackValue(v.to_bits() as _),
    }
  }
}

