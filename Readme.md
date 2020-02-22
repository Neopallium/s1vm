# Structure

* Module - immutable
* Function - immutable
* CodeBlock - immutable
* Instruction - immutable
* VM - mutable

## Module
An Module defines the static code for a program/script.  It contains a list of Functions.
Can have an optional `start` Function that is executed before any exported functions can
be called.

Layout:
* Meta data{version, total size, # globals, # functions, min size required to start execution}
* Export/globals
* One or more Functions.  Allow stream execution.

## Function
* Type
* Locals
* Body - Intructions.

## CodeBlock
* Meta{# locals, # instructions}

## Instruction

* ... many standard ops: Add,Sub,Mul,Load,Store,etc...
* Call - Call another Function inside the Module.
* TailCall - Optimized version of `Call`.
* RustCall - Call non-async Rust function.  Easy to use for API bindings.
* AsyncCall - Async call.

### Call/TailCall
These are optimizations to avoid creating a boxed (heap allocated) future.

### RustCall
For simple library functions.  Don't allow it to call functions in the Module.

### AsyncCall
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

## CallStack
Rust's async support could be used to make the VM stackless without the need for
an internal callstack or C-style coroutine (task/stacklet/etc...).

Stack of:
* FunctionCtx - VM Function context(function idx, pc, locals)
* Future - Rust Future

## VM
The execution context for a Module.  The VM holds an immutable reference to the Module.

## Stack free execution
Loading/executing the Module in a VM should be async.

Load:
* Create a Module by compiling/loading it from a file.  Should support async I/O.
* Create a VM from the Module.  (VM is mutable, Module is immutable).  doesn't block.

Call Function: Rust -> VM
* VM.call(name).  async.  Can yield from

