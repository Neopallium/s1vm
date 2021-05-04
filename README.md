# s1vm

A fast WebAssembly (wasm) interpreter written in 100% safe Rust.

This project started with the idea to port [WASM3](https://github.com/wasm3/wasm3)'s
VM design to safe Rust.

Right now it is a Proof of Concept for trying out some ideas for making a fast VM interpreter in safe Rust (no JIT, no unsafe).

See [Ideas.md](./Ideas.md) for some crazy ideas that might be used.

## VM Design

`s1vm` gets most of it's speed from not using a loop to execute each opcode.  Instead it "compiles" each WASM function using a nest of closure calls.

### Common loop based interpreter

A simple interpreter VM would use a loop and a large switch (match in Rust) to execute opcodes.  That requires multiple jumps (one unpredictable and one predictable jump) for each executed opcode, the unpredictable jump will stall the CPU slowing down execution.

This design also doesn't allow for merging common opcode patterns (GetLocal, I32Const, I32Add).  Merging opcodes can help eliminate alot of stack push/pop operations.

```rust
fn execute_function(func: &Function, state: &State, store: &mut Store) -> Trap<Option<StackValue>> {
  let code = &func.code; // a `Vec<Instruction>` to hold the current function's opcodes.
  let mut pc = 0usize; // PC = program counter.  Index to the next opcode to execute.
  let pc_end = pc + code.len() - 1;

  // loop over the function's opcodes until the function returns or we reach the end of the opcodes.
  loop {
    use crate::isa::Instruction::*; // import opcodes: Br, Return, Call, etc..
    let op = code[pc];
    match op { // The CPU can't predict this jump
      Br(jump_pc) => { // Simple branch opcode (i.e. goto)
        pc = jump_pc as usize;
        continue;
      },
      /* .. other conditional branch opcodes .. */
      Return => {
        if func.ret_type().is_some() {
          // this function returns a value.  Pop it from the stack.
          return Ok(Some(store.stack.pop_val()?));
        } else {
          // no return value.
          return Ok(None);
        }
      },
      Call(func_idx) => {
        // `func_idx` is the unique index for the function that this opcode wants to call.
        if let Some(ret) = state.invoke_function(store, func_idx)? {
          // If the function returns a value, push it onto the stack.
          store.stack.push_val(ret)?;
        }
      },
      // in WASM locals are at fixed-offsets from the top of the stack.
      // They also include the parameters passed on the stack to a function.
      GetLocal(local_idx) => {
        store.stack.get_local(local_idx)?;
      },
      SetLocal(local_idx) => {
        store.stack.set_local(local_idx)?;
      },
      /* .. Get/Set global opcodes. */
      /* .. Load/Store opcodes for reading/writing to memory. */
      /* .. CurrentMemory & GrowMemory opcodes for allocating more memory. */
      // opcodes for pushing i32/i64/f32/f64 constants onto the stack.
      I32Const(val) => store.stack.push(val)?,
      I64Const(val) => store.stack.push(val)?,
      F32Const(val) => store.stack.push(val)?,
      F64Const(val) => store.stack.push(val)?,

      I64Add => {
        let (left, right) = store.stack.pop_pair()? as (i64, i64); // pop 2 values from stack
        let res = left + right; // add the values.
        store.stck.push(res)?;
      },
      /* .. lots of basic i32/i64/f32/f64 opcodes for add/sub/mul/div operations. */
    }
    if pc == pc_end {
      break; // end of function's opcodes.
    }
    pc = pc + 1; // goto next opcode.
  }
  Ok(None)
}
```

### s1vm compiler

The compiler needs to reconstruct the control-flow of each function into one or more Blocks, this will eliminate the need for the `pc` counter during runtime.

When compiling opcodes the compiler keeps a stack of inputs (the outputs of earlier opcodes) that can be consumed by opcodes that pop values from the runtime stack.

As the compiler is processing opcodes, it pushes `Input` values onto the inputs stack.  Opcodes that need inputs will pop them from the inputs stack.  This eliminates a lot of runtime stack push/pops.

Take for example the opcodes for an `add` function:
```
local.get 0 -- Push `Input::Local(0)`
local.get 1 -- Push `Input::Local(1)`
i32.add -- Pop 2 inputs, generate closure for add opcode, push `Input::Op(closure)`
return -- Pop 1 input, generate closure for return opcode and append it to the block.
```

This design allows the compiler to use more specialized code for each opcode based on the types of inputs and eliminated a large amount of runtime stack push/pops.

Compiler types:
```rust
// block type.
enum BlockKind {
  Block,
  Loop,
  If,
  Else,
}

// Action is used for control-flow inside a function's blocks.
enum Action {
  Return(Option<StackValue>), // return from the current function.
  End, // sub-block finish, continue the parent block.
  Branch(u32 /*block depth*/), // return to parent block with the same depth
}

// EvalFunc is for a blocks compiled opcodes.
type EvalFunc = Box<dyn Fn(&State, &mut Store) -> Trap<Action>>;

// OpFunc is for compiled opcodes that have produce a value to be consume as an input.
type OpFunc = Box<dyn Fn(&State, &mut Store) -> Trap<StackValue>>;

enum Input {
  Local(u32 /* local_idx */), // The input is a function local.
  Const(StackValue), // The input is a constant value.
  Op(OpFunc), // The input is the value produced by a compiled opcode.
}

struct Block {
  kind: BlockKind, // block type.  Only used while compiling.
  depth: u32, // this block's depth.
  eval: Vec<EvalFunc>, // vector of compiled sub-blocks and opcodes.
}

```

### s1vm structures

- immutable
  * `State` - Top-level struct that holds the loaded modules.
  * `Module` - Each module has a list of functions, some of those functions are exported to allow other modules or the host to call them.
  * `Function` - Hold the compiled code or bytecode for a function.
- mutable
  * `Store` - Top-level mutable struct that hold the `Memory` and `Stack`.  The `State` can be shared between multiple isolated instanace of the same WASM script.
  * Memory - Just an array of bytes `Vec<u8>`.
  * `Stack` - Holds a stack of values for opcodes that push/pop and for parameter passing when calling a function.  Also helps track the call stack frames.
  * `VM` - Just wraps a `State` and `Store` instance.
- other types
  * `Instruction` - a WASM opcode
  * `StackValue` - wraps a `u64`
  * `Trap` - A specialized `Result` type to handle normal function returns and VM errors (i.e. WASM runtime errors).

## Goals

1. Only use safe Rust.  Crate marked `#![forbid(unsafe_code)]`
2. Support pause/resume.  Either by using `async/await` or stack unwinding/rewinding.
3. Resource limits (CPU/Memory).  Limiting or throttling CPU usage is useful for running sandboxed code.

## Benchmark

Benchmark of `s1vm` against other wasm interpreters:
- [WASM3](https://github.com/wasm3/wasm3) - C
- [wasmi](https://github.com/paritytech/wasmi) - Rust

- fib.wasm 35
  - wasm3 - 0.66 seconds
  - s1vm  - 1.29 seconds
  - wasmi - 3.31 seconds

- fib.wasm 41
  - wasm3 -  9.8 seconds
  - s1vm  - 22.5 seconds
  - wasmi - 57.6 seconds

## TODOs

- [ ] - Support calling host functions.
- [ ] src/compiler.rs - Implement missing compiler opcodes.
