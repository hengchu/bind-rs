use super::bind::*;
use std::collections::VecDeque;

/// Loop over an iterator.
pub fn for_m<'a, M: Monad<'a>, T: 'a, I, F>(xs: I, f: F) -> M::Repr<()>
where
    I: IntoIterator<Item = T>,
    I::IntoIter: Send + 'a,
    F: Fn(T) -> M::Repr<()> + Send + 'a,
{
    let mut it = xs.into_iter();
    match it.next() {
        None => M::ret(()),
        Some(x) => f(x).bind(move |_| for_m::<M, T, _, _>(it, f)),
    }
}

/// Loop over an iterator, collect results into a `VecDeque`.
pub fn map_m<'a, M: Monad<'a>, T: 'a, I, R: 'a, F>(xs: I, f: F) -> M::Repr<VecDeque<R>>
where
    I: IntoIterator<Item = T>,
    I::IntoIter: Send + 'a,
    F: Fn(T) -> M::Repr<R> + Send + 'a,
    R: Send,
{
    let mut it = xs.into_iter();
    match it.next() {
        None => M::ret(VecDeque::new()),
        Some(x) => f(x).bind(move |r| {
            map_m::<M, T, _, R, _>(it, f).bind(move |mut rs| {
                rs.push_front(r);
                M::ret(rs)
            })
        }),
    }
}
