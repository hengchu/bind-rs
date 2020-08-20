use super::bind::*;
use std::marker::PhantomData;

#[derive(Debug)]
/// A namespace for state monads.
pub struct StateM<St>(PhantomData<St>);

/// The representation of a state monad.
pub struct State<'st, St, T>(Box<dyn FnOnce(St) -> (St, T) + Send + 'st>);

/// MTL-style state monad actions.
pub trait MonadState<'st, St: 'st>: Monad<'st> {
    fn get() -> Self::Repr<St>;
    fn put(st: St) -> Self::Repr<()>;
}

impl<'st, St, T> State<'st, St, T> {
    pub fn run(self, st: St) -> (St, T) {
        (self.0)(st)
    }

    pub fn eval(self, st: St) -> T {
        self.run(st).1
    }
}

impl<'st, St: 'st + Send + Clone> MonadState<'st, St> for StateM<St> {
    #[inline]
    fn get() -> Self::Repr<St> {
        State(Box::new(|st| (st.clone(), st)))
    }

    #[inline]
    fn put(st: St) -> Self::Repr<()> {
        State(Box::new(move |_| (st, ())))
    }
}

impl<'st, St: 'st> Monad<'st> for StateM<St> {
    type Repr<T: 'st> = State<'st, St, T>;

    fn bind_impl<A: 'st, B: 'st, F: 'st>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
    where
        F: FnOnce(A) -> Self::Repr<B> + Send,
    {
        let v_f = v.0;
        State(Box::new(move |st| {
            let (st_, a) = v_f(st);
            let b_repr = f(a);
            (b_repr.0)(st_)
        }))
    }

    fn ret<A: 'st + Send>(v: A) -> Self::Repr<A> {
        State(Box::new(move |st| (st, v)))
    }
}

impl<'st, St: 'st, T: 'st> MonadRepr<'st, StateM<St>> for State<'st, St, T> {
    type Index = T;

    fn bind<B: 'st, F: 'st>(self, f: F) -> <StateM<St> as Monad<'st>>::Repr<B>
    where
        F: FnOnce(Self::Index) -> <StateM<St> as Monad<'st>>::Repr<B> + Send,
    {
        <StateM<St> as Monad<'st>>::bind_impl(self, f)
    }
}
