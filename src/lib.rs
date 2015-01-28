#[cfg(test)]
extern crate test;

#[cfg(test)]
use std::collections::{DList, RingBuf};

type Ix = usize;
const END: usize = -1us;

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    link: [usize; 2],  // prev, next
}

impl<T> Node<T> {
    fn prev(&self) -> Ix { self.link[0] }
    fn next(&self) -> Ix { self.link[1] }
    fn prev_mut(&mut self) -> &mut Ix { &mut self.link[0] }
    fn next_mut(&mut self) -> &mut Ix { &mut self.link[1] }
}

#[derive(Debug)]
pub struct List<T> {
    head: usize,
    tail: usize,
    nodes: Vec<Node<T>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Iter<'a, T: 'a>
{
    head: usize,
    tail: usize,
    nodes: &'a [Node<T>],
    taken: usize,
}

#[derive(Debug)]
pub struct IterMut<'a, T: 'a>
{
    head: usize,
    tail: usize,
    nodes: &'a mut [Node<T>],
    taken: usize,
}

#[derive(Debug)]
pub struct Cursor<'a, T: 'a>
{
    pos: usize,
    list: &'a mut List<T>,
}

impl<T> List<T>
{
    pub fn new() -> Self { List::with_capacity(0) }

    pub fn with_capacity(cap: usize) -> Self
    {
        List{
            head: END, tail: END, nodes: Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize
    {
        self.nodes.len()
    }

    pub fn iter(&self) -> Iter<T>
    {
        Iter {
            nodes: &*self.nodes,
            head: self.head,
            tail: self.tail,
            taken: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T>
    {
        IterMut {
            nodes: &mut *self.nodes,
            head: self.head,
            tail: self.tail,
            taken: 0,
        }
    }

    pub fn cursor(&mut self) -> Cursor<T>
    {
        Cursor {
            pos: self.head,
            list: self,
        }
    }

    pub fn push_front(&mut self, value: T) {
        let index = self.nodes.len();
        let node = Node{value: value, link: [END, self.head]};
        self.nodes.push(node);
        if self.head != END {
            *self.nodes[self.head].prev_mut() = index;
        } else {
            self.tail = index;
        }
        self.head = index;
    }

    pub fn push_back(&mut self, value: T) {
        let index = self.nodes.len();
        let node = Node{value: value, link: [self.tail, END]};
        self.nodes.push(node);
        if self.tail != END {
            *self.nodes[self.tail].next_mut() = index;
        } else {
            self.head = index;
        }
        self.tail = index;
    }

    /// "unlink" the node at idx
    fn prepare_remove(&mut self, idx: usize)
    {
        let prev = self.nodes[idx].prev();
        let next = self.nodes[idx].next();
        match self.nodes.get_mut(prev) {
            None => {}
            Some(n) => *n.next_mut() = next,
        }
        match self.nodes.get_mut(next) {
            None => {}
            Some(n) => *n.prev_mut() = prev,
        }
    }

    /// Update links that point to **moved_index** to point to **free_spot**
    /// instead.
    ///
    /// Update head and tail if they point to moved_index.
    fn prepare_swap(&mut self, free_spot: usize, moved_index: usize)
    {
        if free_spot == moved_index {
            return
        }

        let prev = self.nodes[moved_index].prev();
        let next = self.nodes[moved_index].next();
        match self.nodes.get_mut(prev) {
            None => {}
            Some(n) => *n.next_mut() = free_spot,
        }
        match self.nodes.get_mut(next) {
            None => {}
            Some(n) => *n.prev_mut() = free_spot,
        }
        if self.head == moved_index {
            self.head = free_spot;
        }
        if self.tail == moved_index {
            self.tail = free_spot;
        }
    }

    pub fn pop_front(&mut self) -> Option<T>
    {
        if self.head == END {
            return None
        }
        let h = self.head;
        let new_head = self.nodes[h].next();
        self.prepare_remove(h);
        //println!("{:?}", self);

        self.head = new_head;
        if self.head == END {
            self.tail = END;
        } else {
            let moved_index = self.nodes.len() - 1; // last index moves.
            self.prepare_swap(h, moved_index);
        }
        let removed_node = self.nodes.swap_remove(h);
        Some(removed_node.value)
    }

    pub fn pop_back(&mut self) -> Option<T>
    {
        if self.tail == END {
            return None
        }
        let t = self.tail;
        let new_tail = self.nodes[t].prev();
        self.prepare_remove(t);

        self.tail = new_tail;
        if self.tail == END {
            self.head = END;
        } else {
            let moved_index = self.nodes.len() - 1; // last index moves.
            self.prepare_swap(t, moved_index);
        }
        let removed_node = self.nodes.swap_remove(t);
        Some(removed_node.value)
    }

    /// Reorder internal datastructure into traversal order
    pub fn linearize(&mut self)
    {
        // First label every node by ther index in the prev slot
        let mut head = self.head;
        let mut index = 0;
        while let Some(n) = self.nodes.get_mut(head) {
            *n.prev_mut() = index;
            index += 1;
            head = n.next();
        }

        // sort by index
        self.nodes.sort_by(|a, b| a.prev().cmp(&b.prev()));

        // iterate and re-label in order
        let last = self.len() - 1;
        for (index, node) in self.nodes.iter_mut().enumerate() {
            *node.prev_mut() = if index == 0 { END } else { index - 1};
            *node.next_mut() = if index == last { END } else { index + 1 }
        }
        self.head = 0;
        self.tail = last;
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T>
    {
        match self.nodes.get(self.head) {
            None => None,
            Some(n) => {
                self.taken += 1;
                if self.head == self.tail {
                    self.head = END;
                    self.tail = END;
                } else {
                    self.head = n.next();
                }
                Some(&n.value)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.nodes.len() - self.taken;
        (len, Some(len))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T>
{
    fn next_back(&mut self) -> Option<&'a T>
    {
        match self.nodes.get(self.tail) {
            None => None,
            Some(n) => {
                self.taken += 1;
                if self.head == self.tail {
                    self.head = END;
                    self.tail = END;
                } else {
                    self.tail = n.prev();
                }

                Some(&n.value)
            }
        }
    }
}



impl<'a, T: 'a> Iterator for IterMut<'a, T>
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T>
    {
        match self.nodes.get_mut(self.head) {
            None => None,
            Some(n) => {
                self.taken += 1;
                if self.head == self.tail {
                    self.head = END;
                    self.tail = END;
                } else {
                    self.head = n.next();
                }

                // We cannot in safe rust, derive a &'a mut from &mut self,
                // when the life of &mut self is shorter than 'a.
                //
                // We guarantee that this will not allow two pointers to the same
                // element during the iteration, and use unsafe to extend the life.
                //
                // See http://stackoverflow.com/a/25748645/3616050
                let long_life_value = unsafe {
                    &mut *(&mut n.value as *mut _)
                };
                Some(long_life_value)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.nodes.len() - self.taken;
        (len, Some(len))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T>
{
    fn next_back(&mut self) -> Option<&'a mut T>
    {
        match self.nodes.get_mut(self.tail) {
            None => None,
            Some(n) => {
                self.taken += 1;
                if self.head == self.tail {
                    self.head = END;
                    self.tail = END;
                } else {
                    self.tail = n.prev();
                }

                // See .next() above
                let long_life_value = unsafe {
                    &mut *(&mut n.value as *mut _)
                };
                Some(long_life_value)
            }
        }
    }
}

impl<'a, T: 'a> Cursor<'a, T>
{
    pub fn next(&mut self) -> Option<&mut T>
    {
        match self.list.nodes.get_mut(self.pos) {
            None => None,
            Some(n) => {
                self.pos = n.next();
                Some(&mut n.value)
            }
        }
    }

    pub fn prev(&mut self) -> Option<&mut T>
    {
        if self.pos == self.list.head {
            // jump back from head to one past the end, just like gankro's cursor
            self.pos = END;
            return None;
        }
        let prev = 
            match self.list.nodes.get(self.pos) {
                None => self.list.tail,
                Some(n) => n.prev(),
            };
        match self.list.nodes.get_mut(prev) {
            None => None,
            Some(n) => {
                self.pos = prev;
                Some(&mut n.value)
            }
        }
    }
}

#[bench]
fn push_front_dlist(b: &mut test::Bencher)
{
    b.iter(|| {
        let mut dl = DList::new();
        let N = 1000;
        for _ in (0..N) {
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
        let N = 1000;
        for _ in (0..N) {
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
        let N = 1000;
        for _ in (0..N) {
            l.push_front(test::black_box(1));
        }
        l
    })
}

#[bench]
fn push_front_list_cap(b: &mut test::Bencher)
{
    b.iter(|| {
        let N = 1000;
        let mut l = List::with_capacity(N);
        for _ in (0..N) {
            l.push_front(test::black_box(1));
        }
        l
    })
}


#[test]
fn main() {
    let mut l = List::new();
    let front = l.pop_front();
    assert_eq!(front, None);
    assert_eq!(l.iter().count(), 0);
    l.push_back(1);
    assert_eq!(l.iter().count(), 1);
    l.push_front(2);
    assert_eq!(l.iter().count(), 2);
    l.push_front(3);
    assert_eq!(l.iter().count(), 3);
    l.push_back(4);
    assert_eq!(l.iter().count(), 4);
    println!("{:?}", l);
    let front = l.pop_front();
    assert_eq!(front, Some(3));
    println!("{:?}", l);
    assert_eq!(l.iter().count(), 3);

    {
        let mut it = l.iter();
        while let Some(elt) = it.next() {
            println!("Elt={:?}, iter={:?}", elt, it);
        }
    }

    println!("List: {:?}", l.iter().cloned().collect::<Vec<_>>());
    l.pop_back();
    println!("Repr = {:?}", l);
    println!("List: {:?}", l.iter().cloned().collect::<Vec<_>>());
    l.pop_back();
    println!("Repr = {:?}", l);
    println!("List: {:?}", l.iter().cloned().collect::<Vec<_>>());
    l.pop_back();
    println!("Repr = {:?}", l);
    println!("List: {:?}", l.iter().cloned().collect::<Vec<_>>());

    let mut m = List::new();
    m.push_back(2);
    m.push_front(1);
    m.push_back(3);
    m.push_back(4);
    m.push_back(5);

    println!("Repr = {:?}", m);
    println!("List: {:?}", m.iter().cloned().collect::<Vec<_>>());
    m.iter_mut().reverse_in_place();
    println!("Repr = {:?}", m);
    println!("List: {:?}", m.iter().cloned().collect::<Vec<_>>());

    {
        let mut it = m.iter_mut();
        loop {
            match (it.next(), it.next_back()) {
                (Some(a), Some(b)) => {
                    println!("Swap {:?} and {:?}", a, b);
                    std::mem::swap(a, b)
                }
                _ => break,
            }
        }
    }
    println!("Repr = {:?}", m);
    println!("List: {:?}", m.iter().cloned().collect::<Vec<_>>());

    m.linearize();
    println!("Repr = {:?}", m);
    println!("List: {:?}", m.iter().cloned().collect::<Vec<_>>());

    let mut curs = m.cursor();

    curs.prev();
    while let Some(x) = curs.prev() {
        println!("{:?}", x);
    }
}
