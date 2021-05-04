# Ideas

This is to document some of my crazy ideas for this project.

## Threaded interpreter

The normal ways (computed gotos, tail-calls) to optimize VM opcode execution
[do not work in Rust](https://pliniker.github.io/post/dispatchers/).

Crazy ideas:
1. Each opcode as a closure/function.  Still using a loop to call each opcode.
2. Each opcode as a closure that also has a reference to the next opcode.
3. Compile opcodes to nested closures.  All opcodes have a static number of inputs and outputs.

### Compile opcodes into closures

A closure can be made for each opcode of a function to capture the opcode's parameters
(jump target, local/global idx, mem offset, etc..).

It could also allow merging multiple opcodes into a single closure.

Convert (2 push, 2 pop, 1 push):
* GetLocal(0) - push local onto stack.
* I64Const(1234) - push constant '1234' onto stack.
* I64Add - pop two i64 values, push results (a + b)

```rust
/// opcode functions for i64 opcodes.
mod i64_ops {
	fn op_add_local_const(store: &mut Store, local: u32, const_val: i64) -> Trap<()> {
		let left = store.get_local(local);
		let right = const_val;
		let res = left.wrapping_add(right);
		store.push(res);
		Ok(())
	}
	/// many more "merged" opcode functions.....
}
```

The compiler would make a closure:
```rust
let local_idx = 0; // decoded from 'GetLocal(0)' op
let const_val = 1234; // decoded from 'I64Const(1234)' op
let merged_op = move |_state: &State, store: &mut Store| -> Trap<()> {
	i64_ops::op_add_local_const(store, local_idx, const_val);
};
```

## Structure

* Module - immutable
* Function - immutable
* CodeBlock - immutable
* Instruction - immutable
* VM - mutable

### Module
An Module defines the static code for a program/script.  It contains a list of Functions.
Can have an optional `start` Function that is executed before any exported functions can
be called.

Layout:
* Meta data{version, total size, # globals, # functions, min size required to start execution}
* Export/globals
* One or more Functions.  Allow stream execution.

### Function
* Type
* Locals
* Body - Intructions.

### CodeBlock
* Meta{# locals, # instructions}

### Instruction

* ... many standard ops: Add,Sub,Mul,Load,Store,etc...
* Call - Call another Function inside the Module.
* TailCall - Optimized version of `Call`.
* RustCall - Call non-async Rust function.  Easy to use for API bindings.
* AsyncCall - Async call.

#### Call/TailCall
These are optimizations to avoid creating a boxed (heap allocated) future.

#### RustCall
For simple library functions.  Don't allow it to call functions in the Module.

#### AsyncCall
Need to used a `BoxFuture` to avoid recursive async calls.

```rust
use futures::future::{BoxFuture, FutureExt};

fn recursive() -> BoxFuture<'static, ()> {
    async move {
        recursive().await;
        recursive().await;
    }.boxed()
}
```

### CallStack
Rust's async support could be used to make the VM stackless without the need for
an internal callstack or C-style coroutine (task/stacklet/etc...).

Stack of:
* FunctionCtx - VM Function context(function idx, pc, locals)
* Future - Rust Future

### VM
The execution context for a Module.  The VM holds an immutable reference to the Module.

### Stack free execution
Loading/executing the Module in a VM should be async.

Load:
* Create a Module by compiling/loading it from a file.  Should support async I/O.
* Create a VM from the Module.  (VM is mutable, Module is immutable).  doesn't block.

Call Function: Rust -> VM
* VM.call(name).  async.  Can yield from

