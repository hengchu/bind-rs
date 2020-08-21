//! Writer monads can be useful for testing. We can record effectul operations
//! in a trace, and compare an execution trace against an expected trace.

use super::bind::*;
use std::marker::PhantomData;

#[derive(Debug)]
/// A namespace for writer monads.
pub struct WriterM<W>(PhantomData<W>);

/// The representation of a writer monad.
pub struct Writer<W, T> {
    result: T,
    trace: W,
}

/// Abstractions for creating empty traces and appending traces.
pub trait AppendTrace {
    fn empty() -> Self;
    fn append(self, other: Self) -> Self;
}

/// MTL-style writer monad features.
pub trait MonadWriter<'a, W>: Monad<'a> {
    /// Append trace.
    fn write(trace: W) -> Self::Repr<()>;
}

impl<'a, W: AppendTrace> Monad<'a> for WriterM<W> {
    type Repr<T: 'a> = Writer<W, T>;

    fn bind_impl<A: 'a, B: 'a, F: 'a>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        let v_trace = v.trace;
        let v_result = v.result;
        let mut f_repr = f(v_result);
        f_repr.trace = v_trace.append(f_repr.trace);
        f_repr
    }

    fn ret<A: 'a + Send>(v: A) -> Self::Repr<A> {
        Writer {
            result: v,
            trace: W::empty(),
        }
    }
}

impl<'a, W: AppendTrace, T: 'a> MonadRepr<'a, WriterM<W>> for Writer<W, T> {
    type Index = T;

    fn bind<B: 'a, F: 'a>(self, f: F) -> <WriterM<W> as Monad<'a>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <WriterM<W> as Monad<'a>>::Repr<B> + Send,
    {
        <WriterM<W> as Monad<'a>>::bind_impl(self, f)
    }
}
