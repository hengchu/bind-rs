/// A monad is something that we can inject into/bind computations with.
pub trait Monad<'a> {
    /// A `Repr<'a, T>` is a monadic value indexed by type `T`, and is valid for
    /// the given scope (i.e. must be used/evaluated within the specified
    /// scope).

    /// Since the computation produces values of type T, these values must live
    /// for as long as `'a`.
    type Repr<T: 'a>: MonadRepr<'a, Self, Index = T>;

    /// Bind a computation.
    fn bind_impl<A: 'a, B: 'a, F: 'a>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send;

    /// Inject a value into the monad.
    fn ret<A: 'a + Send>(v: A) -> Self::Repr<A>;
}

/// A monad representation is an indexed type that supports the bind operation.
pub trait MonadRepr<'a, M: Monad<'a>> {
    type Index;

    /// Bind a computation, but on the `self` type for more convenient syntax.
    fn bind<B: 'a, F: 'a>(self, f: F) -> M::Repr<B>
    where
        F: FnOnce(Self::Index) -> M::Repr<B> + Send;
}
