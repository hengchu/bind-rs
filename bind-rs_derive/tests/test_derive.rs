use bind_rs::bind::*;
use bind_rs::reader::*;
use bind_rs_derive::Monad;

#[derive(Debug, Monad)]
#[monad(repr = T0Repr<'a, Env, T>)]
struct T0M;
struct T0Repr<'a, Env, T>(Reader<'a, Env, T>);
