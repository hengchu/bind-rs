//! After some experiments, it seems impossible to properly express monad
//! transformers. Hence, we build concrete stacks of monads that form useful
//! patterns. This module implements something akin to `ReaderT IO` in Haskell,
//! but also with async capabilities.

use super::bind::*;
use super::future::MonadFuture;
use super::reader::MonadReader;
use futures::{
    future::{self, BoxFuture},
    Future, FutureExt,
};
use std::marker::PhantomData;

#[derive(Debug)]
/// A namespace for the composition of a reader monad with a future monad.
pub struct ReaderFutureM<Env>(PhantomData<Env>);

/// The representation of reader future monad.
pub struct ReaderFuture<'env, Env, T>(
    Box<dyn FnOnce(&'env Env) -> BoxFuture<'env, T> + Send + 'env>,
);

impl<'env, Env, T> ReaderFuture<'env, Env, T> {
    pub async fn run(self, env: &'env Env) -> T {
        (self.0)(env).await
    }
}

impl<'env, Env: 'env + Sync + Send> MonadReader<'env, Env> for ReaderFutureM<Env> {
    #[inline]
    fn ask_ref() -> Self::Repr<&'env Env> {
        ReaderFuture(Box::new(|env_ref| future::ready(env_ref).boxed()))
    }

    #[inline]
    fn local<R: 'env, F: 'env>(f: F, a: Self::Repr<R>) -> Self::Repr<R>
    where
        F: for<'a> FnOnce(&'a Env) -> &'a Env + Send,
    {
        ReaderFuture(Box::new(move |env_ref| {
            async move { (a.0)(f(env_ref)).await }.boxed()
        }))
    }

    #[inline]
    fn ask() -> Self::Repr<Env>
    where
        Env: Clone,
    {
        ReaderFuture(Box::new(|env_ref| future::ready(env_ref.clone()).boxed()))
    }
}

impl<'env, Env: 'env + Sync> MonadFuture<'env> for ReaderFutureM<Env> {
    fn lift_future<T, Fut: Future<Output = T> + Send + 'env>(fut: Fut) -> Self::Repr<T> {
        ReaderFuture(Box::new(move |_| fut.boxed()))
    }
}

impl<'env, Env: 'env + Sync> Monad<'env> for ReaderFutureM<Env> {
    type Repr<T: 'env> = ReaderFuture<'env, Env, T>;

    fn bind_impl<A: 'env, B: 'env, F: 'env>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        let v_f = v.0;
        ReaderFuture(Box::new(move |env_ref| {
            v_f(env_ref).then(move |a| (f(a).0)(env_ref)).boxed()
        }))
    }

    fn ret<A: 'env + Send>(v: A) -> Self::Repr<A> {
        ReaderFuture(Box::new(move |_| future::ready(v).boxed()))
    }
}

impl<'env, Env: 'env + Sync, T: 'env> MonadRepr<'env, ReaderFutureM<Env>>
    for ReaderFuture<'env, Env, T>
{
    type Index = T;

    fn bind<B: 'env, F: 'env>(self, f: F) -> <ReaderFutureM<Env> as Monad<'env>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <ReaderFutureM<Env> as Monad<'env>>::Repr<B> + Send,
    {
        <ReaderFutureM<Env> as Monad<'env>>::bind_impl(self, f)
    }
}
