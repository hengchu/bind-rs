extern crate bind_rs;
use bind_rs::bind::*;
use bind_rs::reader::*;
use bind_rs::reader_writer::ReaderWriterM;
use bind_rs::writer::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Debug)]
struct Transfer {
    from: String,
    to: String,
    amount: i64,
    from_val: i64,
    to_val: i64,
}

type Book = RefCell<BTreeMap<String, i64>>;

trait MonadBalance<'a>: Monad<'a> {
    fn transfer(from: String, to: String, amount: i64) -> Self::Repr<()>;
}

impl<'a, M: Monad<'a>> MonadBalance<'a> for M
where
    M: MonadReader<'a, Book> + MonadWriter<'a, Vec<Transfer>>,
{
    fn transfer(from: String, to: String, amount: i64) -> Self::Repr<()> {
        Self::ask_ref().bind(move |book_refcell| {
            let mut book_refmut = book_refcell.borrow_mut();
            *book_refmut.entry(from.clone()).or_insert(0) -= amount;
            let from_val = *book_refmut.get(&from).expect("must exist");
            *book_refmut.entry(to.clone()).or_insert(0) += amount;
            let to_val = *book_refmut.get(&to).expect("must exist");
            Self::write(vec![Transfer {
                from,
                to,
                amount,
                from_val,
                to_val,
            }])
        })
    }
}

fn some_transfers<'a, M: MonadBalance<'a>>() -> M::Repr<()> {
    M::transfer("Alice".to_string(), "Bob".to_string(), 100)
        .bind(|_| M::transfer("Bob".to_string(), "Eve".to_string(), 10))
        .bind(|_| M::transfer("Eve".to_string(), "Alice".to_string(), 5))
}

fn main() {
    let storage: Book = RefCell::new(
        vec![("Alice".to_string(), 1000), ("Bob".to_string(), 500)]
            .into_iter()
            .collect(),
    );
    let (trace, _) = some_transfers::<'_, ReaderWriterM<Book, Vec<Transfer>>>().run(&storage);
    println!("transfer history: {:#?}", trace);
    println!("storage: {:#?}", storage);
}
