# bind-rs
![Tests](https://github.com/hengchu/bind-rs/workflows/Tests/badge.svg)
A crate for experimenting with MTL-style monads in rust

### Why?
Monad polymorphism allows us to describe computations without specifying a
concrete interpretation first. This can be useful, for example, if you ever want
to overload a function so that it works for both synchronous code and
async/await code in rust.

### How?
Due to the lack of HKTs in rust, the closest monad implementation we can borrow
from is OCaml's. This crate spiritually follows the OCaml monad design, but uses
rust traits instead of modules.

Basically, to declare to the type system that some type `T` is a monad, we need
to first create a type `TM` that acts as a "namespace" for the monad `T`. Then, the
namespace and the actual monad representation `T` are connected through generic
associated types (at the moment, still an incomplete feature of `rustc`).

For details, check out any of the `src/identity.rs`, `src/future.rs` or
`src/state.rs`, etc.

### Examples
Check out the small testsuite in `src/lib.rs`

Also checkout `examples/balance.rs`, which implements an example of where a
`ReaderWriter` monad stack may be used to hold some state inside a structure
with interior mutability in the reader environment, and uses the writer monad to
record all operations performed on the environment.
