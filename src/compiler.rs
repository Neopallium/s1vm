
use crate::*;
use crate::function::*;
use crate::error::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockKind {
  Block,
  Loop,
  If,
  Else,
}

#[derive(Debug, Clone)]
pub enum Action {
  Return(Option<StackValue>),
  End,
  Branch(u32),
}

type Local = u32;

type EvalFunc = Box<dyn Fn(&vm::State, &mut Store, &mut StackValue) -> Trap<Action>>;

type OpFunc = Box<dyn Fn(&vm::State, &mut Store, &mut StackValue) -> Trap<StackValue>>;

enum Input {
  Local(Local),
  Const(StackValue),
  Op(OpFunc),
}

impl Input {
  pub fn resolv(&self, state: &vm::State, store: &mut Store, l0: &mut StackValue) -> Trap<StackValue> {
    match self {
      Input::Local(0) => {
        Ok(*l0)
      },
      Input::Local(local_idx) => {
        Ok(store.stack.get_local_val(*local_idx, l0))
      },
      Input::Const(const_val) => {
        Ok(*const_val)
      },
      Input::Op(closure) => closure(state, store, l0),
    }
  }
}

pub trait Eval {
  fn eval(&self, state: &vm::State, store: &mut Store, l0: &mut StackValue) -> Trap<StackValue>;
}

impl Eval for Local {
  fn eval(&self, _state: &vm::State, store: &mut Store, l0: &mut StackValue) -> Trap<StackValue> {
    Ok(store.stack.get_local_val(*self, l0))
  }
}

impl Eval for StackValue {
  fn eval(&self, _state: &vm::State, _store: &mut Store, _l0: &mut StackValue) -> Trap<StackValue> {
    Ok(*self)
  }
}

impl Eval for OpFunc {
  fn eval(&self, state: &vm::State, store: &mut Store, l0: &mut StackValue) -> Trap<StackValue> {
    self(state, store, l0)
  }
}

struct Block
{
  kind: BlockKind,
  depth: u32,
  eval: Vec<EvalFunc>,
}

impl Block {
  pub fn new(kind: BlockKind, depth: u32) -> Self {
    Self {
      kind,
      depth,
      eval: vec![],
    }
  }

  pub fn depth(&self) -> u32 {
    self.depth
  }

  pub fn push(&mut self, f: EvalFunc) {
    self.eval.push(f);
  }

  pub fn run(&self, state: &vm::State, store: &mut Store, _l0: &mut StackValue) -> Trap<Action> {
    //eprintln!("---- run block: {:?}, len={}, depth={}", self.kind, self.eval.len(), self.depth);
    'repeat: loop {
      for f in self.eval.iter() {
        let ret = f(state, store, _l0)?;
        //eprintln!("---- evaled: ret = {:?}", ret);
        match ret {
          Action::Return(_) => {
            // Keep passing return value up, until we get to the function block.
            return Ok(ret);
          },
          Action::End => {
            // sub-block finished, continue this block.
            continue;
          },
          Action::Branch(depth) => {
            //eprintln!("---- Branch({})", depth);
            if depth > 0 {
              // keep passing action lower.
              return Ok(Action::Branch(depth-1));
            } else {
              // handle Branch here.
              if self.kind == BlockKind::Loop {
                // Repeat loop block.
                continue 'repeat;
              } else {
                // Normal block, If, or Else.  Just exit on branch.
                return Ok(Action::End);
              }
            }
          }
        }
      }
      // End of block.
      return Ok(Action::End);
    }
  }
}

pub struct State {
  values: Vec<Input>,
  pub depth: u32,
  pub pc: usize,
}

impl State {
  pub fn new() -> Self {
    Self {
      values: vec![],
      depth: 0,
      pc: 0,
    }
  }

  fn pop(&mut self) -> Result<Input> {
    self.values.pop()
      .ok_or_else(|| {
        Error::ValidationError(format!("Value stack empty"))
      })
  }

  fn pop_n(&mut self, n: usize) -> Result<Vec<Input>> {
    let at = self.values.len().checked_sub(n)
      .ok_or_else(|| {
        Error::ValidationError(format!("Value stack empty"))
      })?;
    Ok(self.values.split_off(at))
  }

  fn push(&mut self, input: Input) {
    self.values.push(input);
  }

  fn len(&self) -> usize {
    self.values.len()
  }
}

macro_rules! impl_ops_match_input {
  ($name:ident, $op:expr) => {
    {
      match $name {
        Input::Local($name) => {
          $op
        },
        Input::Const($name) => {
          $op
        },
        Input::Op($name) => {
          $op
        }
      }
    }
  };
}

macro_rules! impl_unops_match_input {
  ($state:ident, $left:ident, $op:expr) => {
    {
      let left = $state.pop()?;
      $state.push(Input::Op(impl_ops_match_input!(left, {
        Box::new(move |state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<StackValue> {
          let $left = left.eval(state, store, l0)?.0;
          let res = $op;
          Ok(StackValue(res as _))
        })
      })));
    }
  };
  ($state:ident, $vm_state:ident, $store:ident, $l0:ident, $left:ident, $op:expr) => {
    {
      let left = $state.pop()?;
      $state.push(Input::Op(impl_ops_match_input!(left, {
        Box::new(move |$vm_state: &vm::State, $store: &mut Store, $l0: &mut StackValue| -> Trap<StackValue> {
          let $left = left.eval($vm_state, $store, $l0)?.0;
          let res = $op;
          Ok(StackValue(res as _))
        })
      })));
    }
  };
}

macro_rules! impl_binops_match_input {
  ($state:ident, $left:ident, $right:ident, $op:expr) => {
    {
      let right = $state.pop()?;
      let left = $state.pop()?;
      $state.push(Input::Op(impl_ops_match_input!(left, {
        impl_ops_match_input!(right, {
          Box::new(move |state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<StackValue> {
            let $left = left.eval(state, store, l0)?.0;
            let $right = right.eval(state, store, l0)?.0;
            let res = $op;
            Ok(StackValue(res as _))
          })
        })
      })));
    }
  };
}

pub struct Compiler {
  module: bwasm::Module,
  compiled: Vec<Function>,

  func_idx: u32,
  ret_type: Option<ValueType>,
  code: Vec<bwasm::Instruction>,
  pc_end: usize,
}

impl Compiler {
  pub fn new(module: &bwasm::Module) -> Self {
    Self {
      module: module.clone(),
      compiled: vec![],

      func_idx: 0,
      ret_type: None,
      code: vec![],
      pc_end: 0,
    }
  }

  pub fn compile(mut self) -> Result<Vec<Function>> {
    let len = self.module.functions().len() as u32;
    for idx in 0..len {
      self.compile_function(idx)?;
    }
    Ok(self.compiled)
  }

  fn compile_function(&mut self, func_idx: u32) -> Result<()> {
    self.func_idx = func_idx;
    let func = self.module.get_func(func_idx)
      .ok_or(Error::FuncNotFound)?;

    if func.is_imported() {
      return Ok(());
    }
    // Compile function into a closure
    self.code = func.instructions().to_vec();
    self.ret_type = func.return_type().map(ValueType::from);
    self.pc_end = self.code.len();

    let mut state = State::new();
    let block = self.compile_block(&mut state, BlockKind::Block)?;

    self.compiled.push(Function::new(func,
    Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Option<StackValue>>
    {
      match block.run(state, store, _l0)? {
        Action::Return(ret_value) => {
          //eprintln!("--- Function return: {:?}", ret_value);
          return Ok(ret_value);
        },
        _ => {
          unreachable!("Compiled function missing 'Return' action.");
        },
      }
    })));

    //eprintln!("---------- depth = {}, values = {}", state.depth, state.len());
    Ok(())
  }

  fn compile_block(&self, state: &mut State, kind: BlockKind) -> Result<Block> {
    let mut block = Block::new(kind, state.depth);
    //eprintln!("compile block: depth: {} {:?}, stack: {}", block.depth(), kind, state.len());
    state.depth += 1;
    if state.depth > 4 {
      panic!("compile overflow, increase the max depth if needed");
    }
    // compile function opcodes.
    loop {
      use parity_wasm::elements::Instruction::*;
      if state.pc > self.pc_end {
        break;
      }
      let pc = state.pc;
      let op = &self.code[pc];
      //eprintln!("compile {}: {:?}", pc, op);
      match op {
        Block(_) => {
          state.pc += 1;
          let sub_block = self.compile_block(state, BlockKind::Block)?;
          block.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
            sub_block.run(state, store, _l0)
          }));
        },
        Loop(_) => {
          state.pc += 1;
          let loop_block = self.compile_loop(state)?;
          block.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
            loop_block.run(state, store, _l0)
          }));
        },
        If(_) => {
          state.pc += 1;
          self.compile_if(&mut block, state)?;
        },
        Else => {
          match kind {
            BlockKind::If => {
              break;
            },
            _ => {
              return Err(Error::ValidationError(format!("invalid 'else' block, missing 'if'")));
            },
          }
        },
        End => {
          if block.depth() == 0 && state.len() > 0 {
            self.emit_return(state, &mut block)?;
          }
          break;
        },
        Return => {
          self.emit_return(state, &mut block)?;
        },
        Br(block_depth) => {
          self.compile_br(&mut block, *block_depth)?;
        },
        BrIf(block_depth) => {
          self.compile_br_if(&mut block, state, *block_depth)?;
        },
        BrTable(ref _br_table) => {
          todo!("");
        },

        Call(func_idx) => {
          let func_idx = *func_idx;
          let func = self.module.get_func(func_idx)
            .ok_or(Error::FuncNotFound)?;
          let count = func.param_count();
          //eprintln!("Call: params={}", count);
          if count > 1 {
            let params = state.pop_n(count as usize - 1)?;
            impl_unops_match_input!(state, vm_state, store, l0, val, {
              let mut val = StackValue(val);
              store.stack.push_val(val)?;
              // Resolve inputs and ..
              let params = params.iter().map(|p| {
                p.resolv(vm_state, store, l0)
              }).collect::<Result<Vec<_>, _>>()?;
              // .. push the values onto the stack.
              store.stack.push_values(&params[..])?;
              if let Some(ret) = vm_state.invoke_function(store, func_idx, &mut val)? {
                ret.0
              } else {
                0
              }
            });
          } else {
            impl_unops_match_input!(state, vm_state, store, l0, val, {
              let mut val = StackValue(val);
              if let Some(ret) = vm_state.invoke_function(store, func_idx, &mut val)? {
                ret.0
              } else {
                0
              }
            });
          }
        },

        GetLocal(local_idx) => {
          state.push(Input::Local(*local_idx));
        },
        SetLocal(set_idx) => {
          let set_idx = *set_idx;
          let val = state.pop()?;
          if set_idx == 0 {
            match val {
              Input::Local(0) => {
                // noop.  Get local 0 and set local 0.
              },
              Input::Local(local_idx) => {
                block.push(Box::new(move |_state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  let val = store.stack.get_local_val(local_idx, l0);
                  *l0 = val;
                  Ok(Action::End)
                }));
              },
              Input::Const(const_val) => {
                block.push(Box::new(move |_state: &vm::State, _store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  *l0 = const_val;
                  Ok(Action::End)
                }));
              },
              Input::Op(closure) => {
                block.push(Box::new(move |state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  *l0 = closure(state, store, l0)?;
                  Ok(Action::End)
                }));
              },
            }
          } else {
            block.push(match val {
              Input::Local(0) => {
                Box::new(move |_state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  let val = *l0;
                  store.stack.set_local_val(set_idx, val, l0);
                  Ok(Action::End)
                })
              },
              Input::Local(local_idx) => {
                Box::new(move |_state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  let val = store.stack.get_local_val(local_idx, l0);
                  store.stack.set_local_val(set_idx, val, l0);
                  Ok(Action::End)
                })
              },
              Input::Const(const_val) => {
                Box::new(move |_state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  let val = const_val;
                  store.stack.set_local_val(set_idx, val, l0);
                  Ok(Action::End)
                })
              },
              Input::Op(closure) => {
                Box::new(move |state: &vm::State, store: &mut Store, l0: &mut StackValue| -> Trap<Action> {
                  let val = closure(state, store, l0)?;
                  store.stack.set_local_val(set_idx, val, l0);
                  Ok(Action::End)
                })
              },
            });
          }
        },
        TeeLocal(set_idx) => {
          let set_idx = *set_idx;
          impl_unops_match_input!(state, vm_state, store, l0, val, {
            store.stack.set_local_val(set_idx, StackValue(val), l0);
            val
          });
        },
        I32Const(val) => {
          state.push(Input::Const(StackValue(*val as _)));
        },
        I64Const(val) => {
          state.push(Input::Const(StackValue(*val as _)));
        },

        I32Add => i32_ops::add(state)?,
        I32Sub => i32_ops::sub(state)?,
        I32LtS => i32_ops::lt_s(state)?,
        I32Eq => i32_ops::eq(state)?,
        I32Eqz => i32_ops::eqz(state)?,

        I64Add => i64_ops::add(state)?,
        I64Sub => i64_ops::sub(state)?,
        I64LtS => i64_ops::lt_s(state)?,
        I64Eq => i64_ops::eq(state)?,
        I64Eqz => i64_ops::eqz(state)?,
        op => todo!("implment opcode: {:?}", op),
      };
      state.pc += 1;
    }

    state.depth -= 1;
    //eprintln!("end block: depth: {} {:?}, stack: {}", block.depth(), kind, state.len());
    Ok(block)
  }

  fn emit_return(&self, state: &mut State, block: &mut Block) -> Result<()> {
    if self.ret_type.is_some() {
      let ret = state.pop()?;
      match ret {
        Input::Local(local_idx) => {
          block.push(Box::new(move |_state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
            let ret = store.stack.get_local_val(local_idx, _l0);
            Ok(Action::Return(Some(StackValue(ret.0 as _))))
          }));
        },
        Input::Const(const_val) => {
          block.push(Box::new(move |_state: &vm::State, _store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
            let ret = const_val;
            Ok(Action::Return(Some(StackValue(ret.0 as _))))
          }));
        },
        Input::Op(closure) => {
          block.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
            let ret = closure(state, store, _l0)?;
            Ok(Action::Return(Some(StackValue(ret.0 as _))))
          }));
        },
      }
    } else {
      block.push(Box::new(move |_state: &vm::State, _store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
        //eprintln!("--- run compiled RETURN: no value");
        Ok(Action::Return(None))
      }));
    }
    Ok(())
  }

  fn compile_loop(&self, state: &mut State) -> Result<Block> {
     self.compile_block(state, BlockKind::Loop)
  }

  fn compile_br(&self, block: &mut Block, block_depth: u32) -> Result<()> {
    //eprintln!("emit br: {:?}", block_depth);
     block.push(Box::new(move |_state: &vm::State, _store: &mut Store, _l0: &mut StackValue| -> Trap<Action> {
       Ok(Action::Branch(block_depth))
     }));
     Ok(())
  }

  fn compile_br_if(&self, block: &mut Block, state: &mut State, block_depth: u32) -> Result<()> {
    //eprintln!("emit br_if: {:?}", block_depth);
    // pop condition value.
    let val = state.pop()?;
    match val {
      Input::Op(closure) => {
        block.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action>
        {
          let val = closure(state, store, _l0)?;
          if val.0 != 0 {
            //eprintln!("branch: {:?}", val);
            Ok(Action::Branch(block_depth))
          } else {
            //eprintln!("continue: {:?}", val);
            Ok(Action::End)
          }
        }));
      },
      _ => {
        block.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action>
        {
          let val = val.resolv(state, store, _l0)?;
          if val.0 != 0 {
            //eprintln!("branch: {:?}", val);
            Ok(Action::Branch(block_depth))
          } else {
            //eprintln!("continue: {:?}", val);
            Ok(Action::End)
          }
        }));
      },
    }
    Ok(())
  }

  fn compile_if(&self, parent: &mut Block, state: &mut State) -> Result<()> {
    // pop condition value.
    let val = state.pop()?;

    // compile 'If' block.
    let if_block = self.compile_block(state, BlockKind::If)?;

    // Check for Else block
    use parity_wasm::elements::Instruction::*;
    let else_block = match &self.code[state.pc] {
      Else => {
        Some(self.compile_else(state)?)
      },
      End => {
        None
      },
      _ => {
        unreachable!("missing end of 'If' block");
      }
    };

    // Build closure.
    if let Some(else_block) = else_block {
      match val {
        Input::Op(closure) => {
          parent.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action>
          {
            let val = closure(state, store, _l0)?;
            if val.0 == 0 {
              else_block.run(state, store, _l0)
            } else {
              if_block.run(state, store, _l0)
            }
          }));
        },
        _ => {
          parent.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action>
          {
            let val = val.resolv(state, store, _l0)?;
            if val.0 == 0 {
              else_block.run(state, store, _l0)
            } else {
              if_block.run(state, store, _l0)
            }
          }));
        },
      }
    } else {
      match val {
        Input::Op(closure) => {
          parent.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action>
          {
            let val = closure(state, store, _l0)?;
            if val.0 == 0 {
              Ok(Action::End)
            } else {
              if_block.run(state, store, _l0)
            }
          }));
        },
        _ => {
          parent.push(Box::new(move |state: &vm::State, store: &mut Store, _l0: &mut StackValue| -> Trap<Action>
          {
            let val = val.resolv(state, store, _l0)?;
            if val.0 == 0 {
              Ok(Action::End)
            } else {
              if_block.run(state, store, _l0)
            }
          }));
        },
      }
    }
    Ok(())
  }

  fn compile_else(&self, state: &mut State) -> Result<Block> {
    self.compile_block(state, BlockKind::Else)
  }
}

macro_rules! impl_int_binops {
  ($name: ident, $type: ty, $op: ident) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_binops_match_input!(state, left, right, {
        (left as $type).$op(right as $type)
      });
      Ok(())
    }
  };
  ($name: ident, $type: ty, $op: ident, $as_type: ty) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_binops_match_input!(state, left, right, {
        (left as $type).$op(right as $type) as $as_type
      });
      Ok(())
    }
  };
  ($name: ident, $type: ty, $type2: ty, $op: ident, $as_type: ty) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_binops_match_input!(state, left, right, {
        (left as $type).$op(right as $type2) as $as_type
      });
      Ok(())
    }
  };
  ($name: ident, $type: ty, $op: ident, $as_type: ty, $mask: expr) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_binops_match_input!(state, left, right, {
        let right = (right as $type) & $mask;
        (left as $type).$op(right as u32) as $as_type
      });
      Ok(())
    }
  };
}

macro_rules! impl_int_binops_div {
  ($name: ident, $type: ty, $op: ident, $as_type: ty) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_binops_match_input!(state, left, right, {
        let res = (left as $type).$op(right as $type)
          .ok_or_else(|| {
            if (right as $type) == 0 {
              TrapKind::DivisionByZero
            } else {
              TrapKind::InvalidConversionToInt
            }
          })?;
        (res as $as_type)
      });
      Ok(())
    }
  };
}

macro_rules! impl_int_relops {
  ($name: ident, $type: ty, $relop: expr) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_unops_match_input!(state, left, {
        $relop(left as $type)
      });
      Ok(())
    }
  };
  ($name: ident, $type: ty, $type2: ty, $relop: expr) => {
    pub fn $name(state: &mut State) -> Result<()> {
      impl_binops_match_input!(state, left, right, {
        $relop(left as $type, right as $type2)
      });
      Ok(())
    }
  };
}

macro_rules! impl_numeric_ops {
  ($op_mod: ident, $type: ty, $type_u: ty) => {
    #[allow(dead_code)]
    mod $op_mod {
      use std::ops::*;
      use super::*;

      pub fn load(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load8_s(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load8_u(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load16_s(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load16_u(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load32_s(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn load32_u(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn store(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn store8(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn store16(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }
      pub fn store32(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }

      impl_int_relops!(clz, $type, |val: $type| {
        val.leading_zeros()
      });
      impl_int_relops!(ctz, $type, |val: $type| {
        val.trailing_zeros()
      });
      impl_int_relops!(popcnt, $type, |val: $type| {
        val.count_ones()
      });

      impl_int_binops!(add, $type, wrapping_add);
      impl_int_binops!(sub, $type, wrapping_sub);

      impl_int_binops!(mul, $type, wrapping_mul);

      impl_int_binops_div!(div_s, $type, checked_div, i64);
      impl_int_binops_div!(div_u, $type, checked_div, u64);
      impl_int_binops_div!(rem_s, $type, checked_rem, i64);
      impl_int_binops_div!(rem_u, $type, checked_rem, u64);

      impl_int_binops!(and, $type, bitand);
      impl_int_binops!(or, $type, bitor);
      impl_int_binops!(xor, $type, bitxor);
      impl_int_binops!(shl, $type, wrapping_shl, $type_u, 0x1F);
      impl_int_binops!(shr_s, $type, wrapping_shr, $type_u, 0x1F);
      impl_int_binops!(shr_u, $type, wrapping_shr, $type_u, 0x1F);
      impl_int_binops!(rotl, $type, u32, rotate_left, u64);
      impl_int_binops!(rotr, $type, u32, rotate_right, u64);

      impl_int_relops!(eqz, $type, |val| {
        val == Default::default()
      });
      impl_int_relops!(eq, $type, $type, |left, right| {
        left == right
      });
      impl_int_relops!(ne, $type, $type, |left, right| {
        left != right
      });
      impl_int_relops!(lt_s, $type, $type, |left, right| {
        left < right
      });
      impl_int_relops!(lt_u, $type_u, $type_u, |left, right| {
        left < right
      });
      impl_int_relops!(gt_s, $type, $type, |left, right| {
        left > right
      });
      impl_int_relops!(gt_u, $type_u, $type_u, |left, right| {
        left > right
      });
      impl_int_relops!(le_s, $type, $type, |left, right| {
        left <= right
      });
      impl_int_relops!(le_u, $type_u, $type_u, |left, right| {
        left <= right
      });
      impl_int_relops!(ge_s, $type, $type, |left, right| {
        left >= right
      });
      impl_int_relops!(ge_u, $type_u, $type_u, |left, right| {
        left >= right
      });

      pub fn trunc_s_f32(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn trunc_u_f32(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn trunc_s_f64(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn trunc_u_f64(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
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

      pub fn load(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn store(_store: &mut Store, _l0: &mut StackValue, _offset: u32) -> Trap<()> {
        todo!();
      }

      pub fn abs(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn neg(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn ceil(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn floor(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn trunc(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn nearest(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn sqrt(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }

      pub fn add(store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        let (left, right) = store.stack.pop_pair()? as ($type, $type);
        let res = left + right;
        store.stack.push(res)?;
        Ok(())
      }

      pub fn sub(store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        let (left, right) = store.stack.pop_pair()? as ($type, $type);
        let res = left - right;
        store.stack.push(res)?;
        Ok(())
      }

      pub fn mul(store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        let (left, right) = store.stack.pop_pair()? as ($type, $type);
        let res = left * right;
        store.stack.push(res)?;
        Ok(())
      }
      pub fn div(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn min(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn max(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn copysign(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }

      pub fn eq(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn ne(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn lt(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn gt(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn le(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn ge(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn convert_s_i32(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn convert_u_i32(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn convert_s_i64(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
      pub fn convert_u_i64(_store: &mut Store, _l0: &mut StackValue) -> Trap<()> {
        todo!();
      }
    }
  };
}

impl_float_numeric_ops!(f32_ops, f32);
impl_float_numeric_ops!(f64_ops, f64);

