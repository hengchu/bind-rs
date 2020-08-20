#![feature(generic_associated_types)]
#![allow(incomplete_features)]
pub mod bind;
pub mod future;
pub mod identity;
pub mod state;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    use super::{bind::*, future::*, identity::*, state::*};
    use futures::executor;

    fn pure<'a, M: Monad<'a>>() -> M::Repr<i64> {
        M::bind_impl(M::bind_impl(M::ret(1), |n| M::ret(n + 1)), |n| {
            M::ret(n * 2)
        })
    }

    fn pure_infix<'a, M: Monad<'a>>() -> M::Repr<i64> {
        M::ret(1).bind(|n| M::ret(n + 1)).bind(|n| M::ret(n * 2))
    }

    #[test]
    fn pure2identity() {
        assert_eq!(pure::<IdentityM>().0, 4);
    }

    #[test]
    fn pure_infix2identity() {
        assert_eq!(pure_infix::<IdentityM>().0, 4);
    }

    #[test]
    fn pure2future() {
        assert_eq!(executor::block_on(pure::<FutureM>()), 4);
    }

    #[test]
    fn pure_infix2future() {
        assert_eq!(executor::block_on(pure_infix::<FutureM>()), 4);
    }

    #[test]
    fn pure2state() {
        let st_repr = pure::<StateM<()>>();
        assert_eq!(st_repr.eval(()), 4);
    }

    #[test]
    fn pure_infix2state() {
        let st_repr = pure_infix::<StateM<()>>();
        assert_eq!(st_repr.eval(()), 4);
    }
}