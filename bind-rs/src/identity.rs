use super::bind::*;

#[derive(Debug)]
pub struct Identity<T>(pub T);

/// A namespace for the `Identity` monad.
pub struct IdentityM;

impl<'a> Monad<'a> for IdentityM {
    type Repr<T: 'a> = Identity<T>;

    fn bind_impl<A: 'a, B: 'a, F: 'a>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        f(v.0)
    }

    fn ret<A: 'a + Send>(v: A) -> Self::Repr<A> {
        Identity(v)
    }
}

impl<'a, T: 'a> MonadRepr<'a, IdentityM> for Identity<T> {
    type Index = T;

    fn bind<B: 'a, F: 'a>(self, f: F) -> <IdentityM as Monad<'a>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <IdentityM as Monad<'a>>::Repr<B> + Send,
    {
        <IdentityM as Monad<'a>>::bind_impl(self, f)
    }
}
