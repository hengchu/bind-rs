use super::bind::*;
use super::reader::*;
use super::writer::*;
use std::marker::PhantomData;

/// A namespace for the reader writer monad.
pub struct ReaderWriterM<Env, W> {
    environment: PhantomData<Env>,
    trace: PhantomData<W>,
}

/// Representation of reader writer monad.
pub struct ReaderWriter<'env, Env, W, T>(Reader<'env, Env, Writer<'env, W, T>>);

impl<'env, Env, W, T> ReaderWriter<'env, Env, W, T> {
    pub fn run(self, env_ref: &'env Env) -> (W, T) {
        let Writer { result, trace, .. } = ((self.0).0)(env_ref);
        (trace, result)
    }
}

impl<'env, Env: 'env, W: 'env + Send + AppendTrace> MonadReader<'env, Env>
    for ReaderWriterM<Env, W>
{
    fn ask_ref() -> Self::Repr<&'env Env> {
        ReaderWriter(Reader(Box::new(|env_ref| Writer {
            lifetime: PhantomData,
            result: env_ref,
            trace: W::empty(),
        })))
    }

    fn local<R: 'env, F: 'env>(f: F, a: Self::Repr<R>) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a Env) -> &'a Env + Send,
    {
        ReaderWriter(Reader(Box::new(move |env_ref| ((a.0).0)(f(env_ref)))))
    }

    fn ask() -> Self::Repr<Env>
    where
        Env: Clone,
    {
        ReaderWriter(Reader(Box::new(|env_ref| Writer {
            lifetime: PhantomData,
            result: env_ref.clone(),
            trace: W::empty(),
        })))
    }
}

impl<'env, Env: 'env, W: 'env + Send + AppendTrace> MonadWriter<'env, W> for ReaderWriterM<Env, W> {
    fn write(trace: W) -> Self::Repr<()> {
        ReaderWriter(Reader(Box::new(move |_| Writer {
            lifetime: PhantomData,
            result: (),
            trace,
        })))
    }
}

impl<'env, Env: 'env, W: 'env + Send + AppendTrace> Monad<'env> for ReaderWriterM<Env, W> {
    type Repr<T: 'env> = ReaderWriter<'env, Env, W, T>;

    fn bind_impl<A: 'env, B: 'env, F: 'env>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        ReaderWriter(Reader(Box::new(move |env_ref| {
            let Writer {
                trace: v_trace,
                result: v_result,
                ..
            } = ((v.0).0)(env_ref);
            let f_repr = f(v_result);
            let Writer {
                trace: f_trace,
                result: f_result,
                ..
            } = ((f_repr.0).0)(env_ref);
            Writer {
                lifetime: PhantomData,
                trace: v_trace.append(f_trace),
                result: f_result,
            }
        })))
    }

    fn ret<A: 'env + Send>(v: A) -> Self::Repr<A> {
        ReaderWriter(ReaderM::<Env>::ret(Writer {
            lifetime: PhantomData,
            result: v,
            trace: W::empty(),
        }))
    }
}

impl<'env, Env: 'env, W: 'env + Send + AppendTrace, T: 'env> MonadRepr<'env, ReaderWriterM<Env, W>>
    for ReaderWriter<'env, Env, W, T>
{
    type Index = T;

    fn bind<B: 'env, F: 'env>(self, f: F) -> <ReaderWriterM<Env, W> as Monad<'env>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <ReaderWriterM<Env, W> as Monad<'env>>::Repr<B> + Send,
    {
        <ReaderWriterM<Env, W> as Monad<'env>>::bind_impl(self, f)
    }
}
