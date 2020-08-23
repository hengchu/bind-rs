use super::bind::*;
use std::marker::PhantomData;

#[derive(Debug)]
/// A name space for reader monads that carry an owned environment.
pub struct ReaderOwnedM<Env>(PhantomData<Env>);

/// The representation of a reader monad that owns the environment.
pub struct ReaderOwned<'env, Env, T>(pub Box<dyn FnOnce(Env) -> T + Send + 'env>);

pub trait MonadReaderOwned<'env, Env: Clone>: Monad<'env> {
    fn ask() -> Self::Repr<Env>;

    fn local<R: 'env, F: 'env>(f: F, a: Self::Repr<R>) -> Self::Repr<R>
    where
        F: FnOnce(Env) -> Env + Send;

    fn local_ref<R: 'env, F: 'env>(f: F) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a Env) -> Self::Repr<R> + Send;

    fn local_mut<R: 'env, F: 'env>(f: F) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a mut Env) -> Self::Repr<R> + Send;
}

impl<'env, Env: 'env + Clone> MonadReaderOwned<'env, Env> for ReaderOwnedM<Env> {
    fn ask() -> Self::Repr<Env> {
        ReaderOwned(Box::new(|env| env))
    }

    fn local<R: 'env, F: 'env>(f: F, a: Self::Repr<R>) -> Self::Repr<R>
    where
        F: FnOnce(Env) -> Env + Send,
    {
        ReaderOwned(Box::new(move |env| (a.0)(f(env))))
    }

    fn local_ref<R: 'env, F: 'env>(f: F) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a Env) -> Self::Repr<R> + Send,
    {
        ReaderOwned(Box::new(move |env| f(&env).0(env)))
    }

    fn local_mut<R: 'env, F: 'env>(f: F) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a mut Env) -> Self::Repr<R> + Send,
    {
        ReaderOwned(Box::new(move |mut env| f(&mut env).0(env)))
    }
}

impl<'env, Env: 'env + Clone> Monad<'env> for ReaderOwnedM<Env> {
    type Repr<T: 'env> = ReaderOwned<'env, Env, T>;

    fn bind_impl<A: 'env, B: 'env, F: 'env>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        let v_f = v.0;
        ReaderOwned(Box::new(move |env| {
            let a = v_f(env.clone());
            f(a).0(env)
        }))
    }

    fn ret<A: 'env + Send>(v: A) -> Self::Repr<A> {
        ReaderOwned(Box::new(move |_| v))
    }
}

impl<'env, Env: 'env + Clone, T: 'env> MonadRepr<'env, ReaderOwnedM<Env>>
    for ReaderOwned<'env, Env, T>
{
    type Index = T;

    fn bind<B: 'env, F: 'env>(self, f: F) -> <ReaderOwnedM<Env> as Monad<'env>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <ReaderOwnedM<Env> as Monad<'env>>::Repr<B> + Send,
    {
        <ReaderOwnedM<Env> as Monad<'env>>::bind_impl(self, f)
    }
}
