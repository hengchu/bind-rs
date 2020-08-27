#![feature(generic_associated_types)]
#![allow(incomplete_features)]
pub mod bind;
pub mod control;
pub mod future;
pub mod identity;
pub mod reader;
pub mod reader_future;
pub mod reader_writer;
pub mod state;
pub mod writer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    use super::{
        bind::*, control::*, future::*, identity::*, reader::*, reader_future::*, state::*,
    };
    use futures::executor;
    use futures::lock::Mutex;
    use futures::FutureExt;
    use std::io::Write;
    use std::sync::Arc;

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

    #[test]
    fn pure2readerfuture() {
        let repr = pure::<ReaderFutureM<()>>();
        assert_eq!(executor::block_on(repr.run(&())), 4);
    }

    #[test]
    fn pure_infix2readerfuture() {
        let repr = pure_infix::<ReaderFutureM<()>>();
        assert_eq!(executor::block_on(repr.run(&())), 4);
    }

    #[test]
    fn test_for_m() {
        let v: Vec<u8> = vec![1, 2, 3, 4, 5];
        type E = Arc<Mutex<std::io::Cursor<Vec<u8>>>>;
        let repr = for_m::<ReaderFutureM<E>, _, _, _>(v, |i| {
            ReaderFutureM::<E>::ask().bind(move |cursor_mutex_arc| {
                ReaderFutureM::<E>::lift_future(
                    async move {
                        cursor_mutex_arc
                            .lock()
                            .await
                            .write_all(&[i])
                            .expect("failed to write")
                    }
                    .boxed(),
                )
            })
        });
        let buffer = Vec::new();
        let e = Arc::new(Mutex::new(std::io::Cursor::new(buffer)));
        executor::block_on(repr.run(&e));
        executor::block_on(async move {
            assert_eq!(e.lock().await.get_ref(), &vec![1, 2, 3, 4, 5]);
        });
    }
}
