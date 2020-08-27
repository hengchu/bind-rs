#![feature(generic_associated_types)]
#![allow(incomplete_features)]

use bind_rs::bind::*;
use bind_rs::reader::*;
use bind_rs::writer::*;
use bind_rs::reader_writer::*;
use bind_rs::reader_future::*;
use bind_rs_derive::Monad;

#[derive(Debug, Monad)]
#[monad(repr = T0Repr<'a, T>, via = ReaderM<i64>)]
struct T0M;
struct T0Repr<'a, T>(Reader<'a, i64, T>);

#[derive(Debug, Monad)]
#[monad(repr = T1Repr<'a, T>, via = ReaderWriterM<i64, Vec<i64>>)]
struct T1M;
struct T1Repr<'a, T>(ReaderWriter<'a, i64, Vec<i64>, T>);

#[derive(Debug, Monad)]
#[monad(repr = T2Repr<'a, T>, via = ReaderFutureM<i64>)]
struct T2M;
struct T2Repr<'a, T>(ReaderFuture<'a, i64, T>);

#[derive(Debug, Monad)]
#[monad(repr = T3Repr<'a, T>, via = WriterM<Vec<i64>>)]
struct T3M;
struct T3Repr<'a, T>(Writer<'a, Vec<i64>, T>);

fn assert_monad<'a, M: Monad<'a>>() {}

#[test]
fn test_simple_derive() {
    assert_monad::<T0M>();
    assert_monad::<T1M>();
    assert_monad::<T2M>();
    assert_monad::<T3M>();
}

//#[derive(Debug, Monad)]
//#[monad(repr = T1Repr<'a, E, T>, via = ReaderM<E>)]
//struct T1M<E>;
//struct T1Repr<'a, E, T>(Reader<'a, E, T>);
