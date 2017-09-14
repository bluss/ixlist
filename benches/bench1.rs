#![feature(test)]
extern crate ixlist;
extern crate test;
extern crate rand;

use rand::{Rng};
use std::collections::{LinkedList, VecDeque};
use ixlist::{
    List,
};



#[bench]
fn push_front_dlist(b: &mut test::Bencher)
{
    b.iter(|| {
        let mut dl = LinkedList::new();
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
        let mut l = VecDeque::new();
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
        let mut l = VecDeque::with_capacity(N);
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

#[bench]
fn iterate_dlist(b: &mut test::Bencher)
{
    let mut dl = LinkedList::new();
    let n = 1000;
    let mut rng = rand::weak_rng();
    for _ in (0..n) {
        if rng.gen() {
            dl.push_front(test::black_box(1));
        } else {
            dl.push_back(test::black_box(1));
        }
    }
    b.iter(|| {
        for elt in dl.iter() {
            test::black_box(elt);
        }
    })
}

#[bench]
fn iterate_ringbuf(b: &mut test::Bencher)
{
    let mut dl = VecDeque::new();
    let n = 1000;
    let mut rng = rand::weak_rng();
    // scramble a bit so we get a random access iteration
    for _ in (0..n) {
        if rng.gen() {
            dl.push_front(test::black_box(1));
        } else {
            dl.push_back(test::black_box(1));
        }
    }
    b.iter(|| {
        for elt in dl.iter() {
            test::black_box(elt);
        }
    })
}


#[bench]
fn iterate_list(b: &mut test::Bencher)
{
    let mut dl = List::new();
    let n = 1000;
    let mut rng = rand::weak_rng();
    // scramble a bit so we get a random access iteration
    for _ in (0..n) {
        if rng.gen() {
            dl.push_front(test::black_box(1));
        } else {
            dl.push_back(test::black_box(1));
        }
    }
    b.iter(|| {
        for elt in dl.iter() {
            test::black_box(elt);
        }
    })
}
