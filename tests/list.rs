extern crate ixlist;

use ixlist::{
    List,
    Seek,
};

#[test]
fn basic()
{
    let l: List<i32> = List::new();
    assert_eq!(l.len(), 0);
}

#[test]
fn push_pop()
{
    let mut l = List::new();
    assert_eq!(l.pop_front(), None);
    assert_eq!(l.pop_back(), None);

    l.push_back(1);
    l.push_back(2);
    l.push_back(3);
    assert_eq!(l.len(), 3);
    assert_eq!(l.pop_back(), Some(3));
    assert_eq!(l.pop_front(), Some(1));
    assert_eq!(l.pop_front(), Some(2));
    assert_eq!(l.pop_back(), None);
    assert_eq!(l.pop_front(), None);

    l.push_front(1);
    l.push_front(2);
    l.push_front(3);
    assert_eq!(l.len(), 3);
    assert_eq!(l.pop_back(), Some(1));
    assert_eq!(l.pop_front(), Some(3));
    assert_eq!(l.pop_front(), Some(2));
    assert_eq!(l.pop_back(), None);
    assert_eq!(l.pop_front(), None);
}

#[test]
fn iter()
{
    let mut l = List::new();
    l.push_back(2);
    l.push_front(1);
    l.push_back(3);
    assert_eq!(l.iter().count(), 3);
    assert_eq!(l.iter().rev().count(), 3);
    assert_eq!(l.iter_mut().count(), 3);
    assert_eq!(l.iter_mut().rev().count(), 3);
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![1,2,3]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![3,2,1]);
    assert_eq!(l.iter_mut().cloned().collect::<Vec<_>>(), vec![1,2,3]);
    assert_eq!(l.iter_mut().rev().cloned().collect::<Vec<_>>(), vec![3,2,1]);
}

#[test]
fn cursor()
{
    let mut l = List::new();
    for index in 0..5 {
        l.push_back(index)
    }
    {
        let mut c = l.cursor();
        assert_eq!(c.next(), Some(&mut 0));
        assert_eq!(c.prev(), Some(&mut 0));
        assert_eq!(c.prev(), None);
        assert_eq!(c.prev(), Some(&mut 4));
        c.insert(77);
        assert_eq!(c.next(), Some(&mut 77));
        assert_eq!(c.next(), Some(&mut 4));
    }
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![0, 1, 2, 3, 77, 4]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![4, 77, 3, 2, 1, 0]);

    {
        let mut c = l.cursor();
        c.seek(Seek::Forward(2));
        c.insert(20);
    }
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![0, 1, 20, 2, 3, 77, 4]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![4, 77, 3, 2, 20, 1, 0]);

    {
        let mut c = l.cursor();
        c.seek(Seek::Forward(2));
        c.seek(Seek::Tail);
        c.insert(30);
    }
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![0, 1, 20, 2, 3, 77, 4, 30]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![30, 4, 77, 3, 2, 20, 1, 0]);

    let mut l = List::new();
    {
        let mut c = l.cursor();
        c.insert(0);
        c.seek(Seek::Tail);
        c.insert(1);
        c.seek(Seek::Head);
        c.insert(2);
        c.seek(Seek::Forward(100));
        c.insert(3);
        c.seek(Seek::Backward(1));
        c.insert(4);
    }
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![2, 0, 4, 1, 3]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![3, 1, 4, 0, 2]);

    l.linearize();

    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![2, 0, 4, 1, 3]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![3, 1, 4, 0, 2]);

    // test wrap around with .next()
    let mut l = List::new();
    {
        let mut c = l.cursor();
        c.insert(0);
        c.insert(1);
        assert!(c.next().is_some());
        assert!(c.next().is_some());
        assert!(c.next().is_none());
        assert!(c.next().is_some());
        c.insert(-1);
    }
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![1, -1, 0]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![0, -1, 1]);
}

#[test]
fn extend()
{
    let mut l = List::new();
    l.push_front(1);
    l.extend(2..2);
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![1]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![1]);

    l.extend(2..5);
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    assert_eq!(l.iter().rev().cloned().collect::<Vec<_>>(), vec![4, 3, 2, 1]);
}

#[test]
fn from_iter()
{
    let l: List<_> = (0..5).collect();
    assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
}
