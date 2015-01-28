extern crate ixlist;
extern crate test;

use std::collections::{DList, RingBuf};
use ixlist::{
    List,
};



#[bench]
fn push_front_dlist(b: &mut test::Bencher)
{
    b.iter(|| {
        let mut dl = DList::new();
        let n = 1000;
        for _ in (0..n) {
            dl.push_front(test::black_box(1));
        }
        dl
    })
}

#[bench]
fn push_front_ringbuf(b: &mut test::Bencher)
{
    b.iter(|| {
        let mut l = RingBuf::new();
        let n = 1000;
        for _ in (0..n) {
            l.push_front(test::black_box(1));
        }
        l
    })
}

// benches perform worse if this is included..
/*
#[bench]
fn push_front_ringbuf_cap(b: &mut test::Bencher)
{
    b.iter(|| {
        let N = 1000;
        let mut l = RingBuf::with_capacity(N);
        for _ in (0..N) {
            l.push_front(test::black_box(1));
        }
        l
    })
}
*/

#[bench]
fn push_front_list(b: &mut test::Bencher)
{
    b.iter(|| {
        let mut l = List::new();
        let n = 1000;
        for _ in (0..n) {
            l.push_front(test::black_box(1));
        }
        l
    })
}

#[bench]
fn push_front_list_cap(b: &mut test::Bencher)
{
    b.iter(|| {
        let n = 1000;
        let mut l = List::with_capacity(n);
        for _ in (0..n) {
            l.push_front(test::black_box(1));
        }
        l
    })
}

