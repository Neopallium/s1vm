
use crate::error::*;
use crate::value::*;

pub const DEFAULT_STACK_LIMIT: usize = 1024 * 1024;

#[derive(Debug)]
pub struct StackPtr(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct StackValue(pub u64);

#[derive(Debug, Clone)]
pub struct Stack {
  stack: Vec<StackValue>,
  /// Base Pointer - for the current function's stack frame.
  bp: usize,
  /// Maximum stack size.
  limit: usize,
}

impl Stack {
  pub fn new() -> Stack {
    Self::new_with_limit(DEFAULT_STACK_LIMIT)
  }

  pub fn new_with_limit(limit: usize) -> Stack {
    Stack {
      stack: Default::default(),
      bp: 0,
      limit,
    }
  }

  pub fn len(&self) -> usize {
    self.stack.len()
  }

  /// How many values are on the current frame
  pub fn frame_size(&self) -> Trap<usize> {
    let len = self.stack.len();
    let size = len
      .checked_sub(self.bp)
      .ok_or_else(|| {
        eprintln!("StackOverflow: bp={}, len={}", self.bp, len);
        TrapKind::StackOverflow
      })?;
    Ok(size)
  }

  fn check_overflow(&mut self, need: usize) -> Trap<usize> {
    let len = self.stack.len();
    self.limit
      .checked_sub(len)
      .and_then(|x| x.checked_sub(need))
      .ok_or_else(|| {
        eprintln!("StackOverflow: limit={}, len={}", self.limit, len);
        TrapKind::StackOverflow
      })?;

    // Return the current stack size.
    Ok(len)
  }

  /// Start a new stack frame by saving the current base pointer.
  pub fn push_frame(&mut self, params: usize) -> Trap<StackPtr> {
    // Check if there are enough values on the current stack frame for
    // the new function's parameters.
    let cur_size = self.frame_size()?;
    if cur_size < params {
      eprintln!("StackOverflow: frame size={}, needed params={}", cur_size, params);
      return Err(TrapKind::StackOverflow);
    }

    // save old BP
    let old_bp = self.bp;

    // Include the params in the new stack frame.
    let new_bp = self.stack.len() - params;
    self.bp = new_bp;

    Ok(StackPtr(old_bp))
  }

  pub fn reserve_locals(&mut self, locals: usize) -> Trap<()> {
    // Reserve stack space for the locals.
    if locals > 0 {
      let new_len = self.stack.len()
        .checked_add(locals)
        .ok_or_else(|| {
          eprintln!("StackOverflow: len={}, locals={}", self.stack.len(), locals);
          TrapKind::StackOverflow
        })?;
      self.stack.resize(new_len, StackValue(0));
    }

    Ok(())
  }

  /// Remove current stack frame and restore previous frame.
  pub fn pop_frame(&mut self, old_bp: StackPtr) {
    let StackPtr(old_bp) = old_bp;
    // Drop current stack frame values.
    self.stack.resize(self.bp, StackValue(0));
    // Restore old base pointer.
    self.bp = old_bp;
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
    let len = self.stack.len();
    let new_len = len
      .checked_sub(count as usize)
      .ok_or_else(|| {
        eprintln!("StackOverflow: ({} - {}) < 0", len, count);
        TrapKind::StackOverflow
      })?;
    self.stack.resize(new_len, StackValue(0));
    Ok(())
  }

  #[inline]
  fn check_local(&mut self, local: LocalIdx) -> Trap<usize> {
    // Calculate local index and validate math.
    self.bp
      .checked_add(local as usize)
      .ok_or_else(|| {
        eprintln!("StackOverflow: bp={}, local={}, len={}", self.bp, local, self.stack.len());
        TrapKind::StackOverflow
      })
  }

  #[inline]
  pub fn set_local_val(&mut self, local: LocalIdx, val: StackValue) -> Trap<()> {
    let idx = self.check_local(local)?;

    if let Some(elem) = self.stack.get_mut(idx) {
      *elem = val;
      Ok(())
    } else {
      Err(TrapKind::StackOverflow)
    }
  }

  #[inline]
  pub fn get_local_val(&mut self, local: LocalIdx) -> Trap<StackValue> {
    let idx = self.check_local(local)?;

    if let Some(elem) = self.stack.get(idx) {
      Ok(*elem)
    } else {
      Err(TrapKind::StackOverflow)
    }
  }

  #[inline]
  pub fn push_val(&mut self, val: StackValue) -> Trap<()> {
    if self.stack.len() >=  self.limit {
      eprintln!("StackOverflow: limit={}, len={}", self.limit, self.stack.len());
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
    let val = self.stack.pop()
      .ok_or_else(|| {
        eprintln!("StackOverflow: bp={}, len={}", self.bp, self.stack.len());
        TrapKind::StackOverflow
      });
    //eprintln!("-- Pop: {:?}", val);
    val
  }

  #[inline]
  pub fn top_val(&mut self) -> Trap<StackValue> {
    self.stack.last().map(|x| *x)
      .ok_or_else(|| {
        eprintln!("StackOverflow: bp={}, len={}", self.bp, self.stack.len());
        TrapKind::StackOverflow
      })
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

  fn set_local(&mut self, local: LocalIdx, val: T) -> Trap<()>;
  fn get_local(&mut self, local: LocalIdx) -> Trap<T>;

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

        #[inline]
        fn set_local(&mut self, local: LocalIdx, val: $t) -> Trap<()> {
          self.set_local_val(local, StackValue(val as _))
        }

        #[inline]
        fn get_local(&mut self, local: LocalIdx) -> Trap<$t> {
          self.get_local_val(local).map(|x| x.0 as _)
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

        #[inline]
        fn set_local(&mut self, local: LocalIdx, val: $t) -> Trap<()> {
          self.set_local_val(local, StackValue(val.to_bits() as _))
        }

        #[inline]
        fn get_local(&mut self, local: LocalIdx) -> Trap<$t> {
          self.get_local_val(local).map(|x| <$t>::from_bits(x.0 as _))
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

