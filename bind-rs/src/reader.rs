use super::bind::*;
use std::marker::PhantomData;

#[derive(Debug)]
/// A namespace for reader monads.
pub struct ReaderM<Env>(PhantomData<Env>);

/// The representation of a reader monad.
pub struct Reader<'env, Env, T>(pub Box<dyn FnOnce(&'env Env) -> T + Send + 'env>);

/// MTL-style reader monad actions.
pub trait MonadReader<'env, Env: 'env>: Monad<'env> {
    /// Get a reference to the environment.
    fn ask_ref() -> Self::Repr<&'env Env>;

    /// Run a computation in a modified local environment.
    fn local<R: 'env, F: 'env>(f: F, a: Self::Repr<R>) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a Env) -> &'a Env + Send;

    /// Get an owned clone of the environment.
    fn ask() -> Self::Repr<Env>
    where
        Env: Clone;
}

impl<'env, Env: 'env> MonadReader<'env, Env> for ReaderM<Env> {
    #[inline]
    fn ask_ref() -> Self::Repr<&'env Env> {
        Reader(Box::new(|env_ref| env_ref))
    }

    #[inline]
    fn local<R: 'env, F: 'env>(f: F, a: Self::Repr<R>) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a Env) -> &'a Env + Send,
    {
        Reader(Box::new(|env_ref| (a.0)(f(env_ref))))
    }

    #[inline]
    fn ask() -> Self::Repr<Env>
    where
        Env: Clone,
    {
        Reader(Box::new(|env_ref| env_ref.clone()))
    }
}

impl<'env, Env: 'env> Monad<'env> for ReaderM<Env> {
    type Repr<T: 'env> = Reader<'env, Env, T>;

    fn bind_impl<A: 'env, B: 'env, F: 'env>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        let v_f = v.0;
        Reader(Box::new(move |env| {
            let a = { v_f(&env) };
            f(a).0(&env)
        }))
    }

    fn ret<A: 'env + Send>(v: A) -> Self::Repr<A> {
        Reader(Box::new(move |_| v))
    }
}

impl<'env, Env: 'env, T: 'env> MonadRepr<'env, ReaderM<Env>> for Reader<'env, Env, T> {
    type Index = T;

    fn bind<B: 'env, F: 'env>(self, f: F) -> <ReaderM<Env> as Monad<'env>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <ReaderM<Env> as Monad<'env>>::Repr<B> + Send,
    {
        <ReaderM<Env> as Monad<'env>>::bind_impl(self, f)
    }
}
