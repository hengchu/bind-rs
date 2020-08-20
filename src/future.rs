use super::bind::*;
use futures::{
    future::{self, BoxFuture},
    FutureExt,
};

/// A namespace for the `Future` monad.
pub struct FutureM;

impl<'a> Monad<'a> for FutureM {
    type Repr<T: 'a> = BoxFuture<'a, T>;

    fn bind_impl<A: 'a, B: 'a, F: 'a>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        v.then(f).boxed()
    }

    fn ret<A: 'a + Send>(v: A) -> Self::Repr<A> {
        future::ready(v).boxed()
    }
}

impl<'a, T: 'a> MonadRepr<'a, FutureM> for BoxFuture<'a, T> {
    type Index = T;

    fn bind<B: 'a, F: 'a>(self, f: F) -> <FutureM as Monad<'a>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <FutureM as Monad<'a>>::Repr<B> + Send,
    {
        <FutureM as Monad<'a>>::bind_impl(self, f)
    }
}