extern crate ixlist;
extern crate rand;
#[macro_use] extern crate bencher;

use bencher::Bencher;

use rand::{Rng};
use rand::XorShiftRng;
use std::collections::{LinkedList, VecDeque};
use ixlist::{
    List,
};
use bencher::black_box;

// fn id<T>(t: T) -> T { t }

// reproducible rng
fn repro_rng() -> XorShiftRng { XorShiftRng::new_unseeded() }


fn push_front_dlist(b: &mut Bencher)
{
    b.iter(|| {
        let mut dl = LinkedList::new();
        let n = 1000;
        for _ in (0..n) {
            dl.push_front(black_box(1));
        }
        dl
    })
}

fn push_front_ringbuf(b: &mut Bencher)
{
    b.iter(|| {
        let mut l = VecDeque::new();
        let n = 1000;
        for _ in (0..n) {
            l.push_front(black_box(1));
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
        let mut l = VecDeque::with_capacity(N);
        for _ in (0..N) {
            l.push_front(black_box(1));
        }
        l
    })
}
*/

fn push_front_list(b: &mut Bencher)
{
    b.iter(|| {
        let mut l = List::new();
        let n = 1000;
        for _ in (0..n) {
            l.push_front(black_box(1));
        }
        l
    })
}

fn push_front_list_cap(b: &mut Bencher)
{
    b.iter(|| {
        let n = 1000;
        let mut l = List::with_capacity(n);
        for _ in (0..n) {
            l.push_front(black_box(1));
        }
        l
    })
}

fn iterate_dlist(b: &mut Bencher)
{
    let mut dl = LinkedList::new();
    let n = 1000;
    let mut rng = repro_rng();
    for _ in (0..n) {
        if rng.gen() {
            dl.push_front(black_box(1));
        } else {
            dl.push_back(black_box(1));
        }
    }
    b.iter(|| {
        for elt in dl.iter() {
            black_box(elt);
        }
    })
}

fn iterate_ringbuf(b: &mut Bencher)
{
    let mut dl = VecDeque::new();
    let n = 1000;
    let mut rng = repro_rng();
    // scramble a bit so we get a random access iteration
    for _ in (0..n) {
        if rng.gen() {
            dl.push_front(black_box(1));
        } else {
            dl.push_back(black_box(1));
        }
    }
    b.iter(|| {
        for elt in dl.iter() {
            black_box(elt);
        }
    })
}


fn iterate_list(b: &mut Bencher)
{
    let mut dl = List::new();
    let n = 1000;
    let mut rng = repro_rng();
    // scramble a bit so we get a random access iteration
    for _ in (0..n) {
        if rng.gen() {
            dl.push_front(black_box(1));
        } else {
            dl.push_back(black_box(1));
        }
    }
    b.iter(|| {
        for elt in dl.iter() {
            black_box(elt);
        }
    })
}

benchmark_group!(benches,
                 push_front_dlist,
                 push_front_ringbuf,
                 push_front_list,
                 push_front_list_cap,
                 iterate_dlist,
                 iterate_ringbuf,
                 iterate_list);
benchmark_main!(benches);
