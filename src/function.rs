
use crate::*;
use crate::error::*;

type CompiledFunc = Box<dyn Fn(&mut Store, &mut StackValue) -> Trap<Option<StackValue>>>;

pub struct CompiledFunction {
  pub local_types: Vec<ValueType>,
  pub run: CompiledFunc,
}

#[derive(Debug, Clone)]
pub struct HostFunction {
  pub mod_idx: usize,
  pub func_idx: usize,
}

pub enum FunctionBody {
  Compiled(CompiledFunction),
  Host(HostFunction),
}

pub struct Function {
  pub name: String,
  pub func_type: FunctionType,
  pub body: FunctionBody,
}

impl Function {
  pub fn new(func: &bwasm::Function, run: CompiledFunc) -> Function {
    Function {
      name: func.name().to_string(),
      func_type: FunctionType::from(func.func_type()),
      body: FunctionBody::Compiled(CompiledFunction{
        local_types: ValueType::from_slice(func.locals()),
        run,
      }),
    }
  }

  pub fn param_count(&self) -> usize {
    self.func_type.param_count()
  }

  pub fn ret_type(&self) -> Option<ValueType> {
    self.func_type.ret_type
  }

  pub fn call(&self, store: &mut Store, l0: &mut StackValue) -> Trap<Option<StackValue>> {
    match self.body {
      FunctionBody::Compiled(ref body) => {
        let p_count = self.param_count();
        let l_count = body.local_types.len();
        let ret = if p_count <= 1 && l_count == 0 {
          // Fast function call.
          (body.run)(store, l0)?
        } else {
          // Setup stack frame for function.
          let old_frame = store.stack.push_frame(p_count, l_count)?;

          // run function
          let ret = (body.run)(store, l0)?;

          // cleanup stack frame.
          store.stack.pop_frame(old_frame);
          ret
        };
        Ok(ret)
      },
      FunctionBody::Host(_) => {
        todo!("");
      },
    }
  }
}
