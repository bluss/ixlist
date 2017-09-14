
use std::iter::IntoIterator;
use std::iter::FromIterator;

type Ix = usize;
/// **END** is the "null" pointer of the link indexes
const END: usize = std::usize::MAX;

#[derive(Clone, Debug)]
pub struct Node<T> {
    /// Prev, Next.
    link: [usize; 2],
    pub value: T,
}

impl<T> Node<T> {
    fn new(value: T, prev: Ix, next: Ix) -> Self
    {
        Node {
            value: value,
            link: [prev, next],
        }
    }
    fn prev(&self) -> Ix { self.link[0] }
    fn next(&self) -> Ix { self.link[1] }
    fn set_prev(&mut self, index: Ix) { self.link[0] = index; }
    fn set_next(&mut self, index: Ix) { self.link[1] = index; }
}

/// **List** is a doubly linked list stored in one contiguous allocation.
///
/// ## Features
///
/// * O(1) insert and remove both at front and back.
/// * O(1) insert anywhere if you have a cursor to that position.
/// * Only use of **unsafe** is an unavoidable use for **IterMut**.
///
///
/// ## Implementation
///
/// It is similar to a linked list in a language like C, except instead of pointers we
/// use indices into a backing vector.
///
/// The list is just a vector, and indices to the head and tail:
///
/// ```ignore
/// struct List<T> {
///     /// Head, Tail
///     link: [usize; 2],
///     nodes: Vec<Node<T>>,
/// }
/// ```
///
/// The list node is represented like this:
///
/// ```ignore
/// struct Node<T> {
///     /// Prev, Next.
///     link: [usize; 2],
///     value: T,
/// }
/// ```
///
/// The `link` arrays contain the vector indices of the previous and next node. We
/// use an array so that symmetries in front/back or prev/next can be used easily in the
/// code — it's nice if we can write just one push and one pop method instead of two.
///
/// There is a constant to denote a “null” index, and that's usize's max value.
/// We don't always have to check for this case, we can just access the nodes
/// vector using *.get()* or *.get_mut()*; a “null” link is the **None** case.
///
/// ## To do
///
/// List could be generic over the index type, so that internal
/// prev/node links can use less space than a regular pointer (can be u16 or u32 index).
///
/// With some cleanup we can use unchecked indexing — but it's not guaranteed
/// to make any difference.
///
#[derive(Clone, Debug)]
pub struct List<T> {
    /// Head, Tail
    link: [usize; 2],
    nodes: Vec<Node<T>>,
}

/// Represent one of the two ends of the list
#[derive(Copy, Clone, PartialEq, Debug)]
enum Terminal {
    Head = 0,
    Tail = 1,
}

impl Terminal
{
    #[inline]
    pub fn opposite(&self) -> Self
    {
        match *self {
            Terminal::Head => Terminal::Tail,
            Terminal::Tail => Terminal::Head,
        }
    }

    #[inline]
    pub fn index(&self) -> usize { *self as usize }
}

#[derive(Copy, Clone, Debug)]
pub struct Iter<'a, T: 'a>
{
    link: [usize; 2],
    nodes: &'a [Node<T>],
    taken: usize,
}

#[derive(Debug)]
pub struct IterMut<'a, T: 'a>
{
    link: [usize; 2],
    nodes: &'a mut [Node<T>],
    taken: usize,
}

/// A cursor points to a location in a list, and you can step the
/// cursor forward and backward.
#[derive(Debug)]
pub struct Cursor<'a, T: 'a>
{
    pos: usize,
    list: &'a mut List<T>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Seek {
    /// Seek forward *n* steps, or at most to the end.
    Forward(usize),
    /// Seek backward *n* steps, or at most to the beginning.
    Backward(usize),
    /// Seek to the beginning.
    Head,
    /// Seek to the end.
    Tail,
}

impl<T> List<T>
{
    /// Create a new **List**.
    pub fn new() -> Self { List::with_capacity(0) }

    /// Create a new **List** with specified capacity.
    pub fn with_capacity(cap: usize) -> Self
    {
        List{
            link: [END; 2], nodes: Vec::with_capacity(cap),
        }
    }

    fn head(&self) -> usize { self.link[0] }
    fn tail(&self) -> usize { self.link[1] }

    /// Return the number of elements in the List.
    pub fn len(&self) -> usize
    {
        self.nodes.len()
    }

    /// Return an iterator.
    pub fn iter(&self) -> Iter<T>
    {
        Iter {
            link: self.link,
            nodes: &*self.nodes,
            taken: 0,
        }
    }

    /// Return an iterator.
    pub fn iter_mut(&mut self) -> IterMut<T>
    {
        IterMut {
            link: self.link,
            nodes: &mut *self.nodes,
            taken: 0,
        }
    }

    /// Return a new cursor, focused before the head of the List.
    pub fn cursor(&mut self) -> Cursor<T>
    {
        Cursor {
            pos: self.head(),
            list: self,
        }
    }

    fn push_terminal(&mut self, value: T, term: Terminal)
    {
        let t = term as usize;
        let index = self.nodes.len();
        let mut node = Node::new(value, END, END);
        node.link[1 - t] = self.link[t];

        match self.nodes.get_mut(self.link[t]) {
            None => self.link[1 - t] = index, // List was empty
            Some(n) => n.link[t] = index,
        }
        self.link[t] = index;
        self.nodes.push(node);
    }

    /// Insert an element at the beginning of the List.
    pub fn push_front(&mut self, value: T) {
        self.push_terminal(value, Terminal::Head)
    }

    /// Insert an element at the end of the List.
    pub fn push_back(&mut self, value: T) {
        self.push_terminal(value, Terminal::Tail)
    }

    /// "unlink" the node at idx
    fn prepare_remove(&mut self, idx: usize)
    {
        let prev = self.nodes[idx].prev();
        let next = self.nodes[idx].next();
        match self.nodes.get_mut(prev) {
            None => {}
            Some(n) => n.set_next(next),
        }
        match self.nodes.get_mut(next) {
            None => {}
            Some(n) => n.set_prev(prev),
        }
    }

    /// Change pointers to the node at **idx** to point to **to_index** instead.
    fn prepare_move(&mut self, idx: usize, to_index: usize)
    {
        let prev = self.nodes[idx].prev();
        let next = self.nodes[idx].next();
        match self.nodes.get_mut(prev) {
            None => {}
            Some(n) => n.set_next(to_index),
        }
        match self.nodes.get_mut(next) {
            None => {}
            Some(n) => n.set_prev(to_index),
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

        self.prepare_move(moved_index, free_spot);
        if self.head() == moved_index {
            self.link[0] = free_spot;
        }
        if self.tail() == moved_index {
            self.link[1] = free_spot;
        }
    }

    /// Remove the element at either head or tail
    fn pop_terminal(&mut self, term: Terminal) -> Option<T>
    {
        let t = term as usize;
        if self.link[t] == END {
            return None
        }
        let h = self.link[t];
        let new_terminal = self.nodes[h].link[1 - t];
        self.prepare_remove(h);

        self.link[t] = new_terminal;
        if self.link[t] == END {
            self.link[1 - t] = END;
        } else {
            let moved_index = self.nodes.len() - 1; // last index moves.
            self.prepare_swap(h, moved_index);
        }
        let removed_node = self.nodes.swap_remove(h);
        Some(removed_node.value)
    }

    /// Remove the element at the beginning of the List and return it,
    /// or return **None** if the List is empty.
    pub fn pop_front(&mut self) -> Option<T>
    {
        self.pop_terminal(Terminal::Head)
    }

    /// Remove the element at the end of the List and return it,
    /// or return **None** if the List is empty.
    pub fn pop_back(&mut self) -> Option<T>
    {
        self.pop_terminal(Terminal::Tail)
    }

    /// Reorder internal datastructure into traversal order.
    pub fn linearize(&mut self)
    {
        if self.len() == 0 {
            return;
        }

        // First label every node by their index + 1 in the next slot
        let mut head = self.head();
        let mut index = 0;
        while let Some(n) = self.nodes.get_mut(head) {
            index += 1;
            head = n.next();
            n.set_next(index);
        }

        // sort by index
        self.nodes.sort_unstable_by_key(Node::next);

        // iterate and re-label in order
        // prev's need update, all the next links except the last should be ok.
        for (index, node) in self.nodes[1..].iter_mut().enumerate() {
            node.set_prev(index);
        }
        self.link[0] = 0;
        self.link[1] = self.len() - 1;
        self.nodes[self.link[0]].set_prev(END);
        self.nodes[self.link[1]].set_next(END);
    }
}

impl<'a, T> FromIterator<T> for List<T>
{
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=T>
    {
        let mut result = List::new();
        result.extend(iter);
        result
    }
}

impl<'a, T> Extend<T> for List<T>
{
    fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=T>
    {
        let mut iter = iter.into_iter();
        let (low, _) = iter.size_hint();
        self.nodes.reserve(low);
        let tail = self.tail();
        let index = self.nodes.len();

        // pick the first to set prev to tail
        for elt in iter.by_ref() {
            let node = Node::new(elt, tail, index + 1);
            self.nodes.push(node);
            break;
        }

        for (i, elt) in iter.enumerate() {
            let node = Node::new(elt, index + i, index + i + 2);
            self.nodes.push(node);
        }

        if self.nodes.len() == 0 {
            return;
        }

        match self.nodes.get_mut(self.link[1]) {
            None => self.link[0] = index, // List was empty
            Some(tailn) => tailn.set_next(index),
        }
        self.link[1] = self.nodes.len() - 1;
        self.nodes[self.link[1]].set_next(END);
    }
}

impl<'a, T: 'a> Iter<'a, T>
{
    /// Step the iterator from the head or tail
    fn next_terminal(&mut self, term: Terminal) -> Option<&'a T>
    {
        let h = term.index();
        let t = term.opposite().index();
        match self.nodes.get(self.link[h]) {
            None => None,
            Some(n) => {
                // Extract `elt` already here, to avoid spurious null check for elt
                let elt = Some(&n.value);
                self.taken += 1;
                if self.link[h] == self.link[t] {
                    self.link[0] = END;
                    self.link[1] = END;
                } else {
                    self.link[h] = n.link[t];
                }
                elt
            }
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T>
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> { self.next_terminal(Terminal::Head) }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.nodes.len() - self.taken;
        (len, Some(len))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T>
{
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> { self.next_terminal(Terminal::Tail) }
}


impl<'a, T: 'a> IterMut<'a, T>
{
    /// Step the iterator from the head or tail
    fn next_terminal(&mut self, term: Terminal) -> Option<&'a mut T>
    {
        let h = term.index();
        let t = term.opposite().index();
        match self.nodes.get_mut(self.link[h]) {
            None => None,
            Some(n) => {
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
                let elt = Some(long_life_value);

                self.taken += 1;
                if self.link[h] == self.link[t] {
                    self.link = [END, END];
                } else {
                    self.link[h] = n.link[t];
                }
                elt
            }
        }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T>
{
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> { self.next_terminal(Terminal::Head) }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.nodes.len() - self.taken;
        (len, Some(len))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T>
{
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut T> { self.next_terminal(Terminal::Tail) }
}

impl<'a, T: 'a> Cursor<'a, T>
{
    /// Step the cursor forward.
    /// 
    /// Returns **None** after the last element. After that, another call to
    /// *.next()* returns the first element of the list.
    pub fn next(&mut self) -> Option<&mut T>
    {
        match self.list.nodes.get_mut(self.pos) {
            None => {
                self.pos = self.list.link[0];
                None
            }
            Some(n) => {
                self.pos = n.next();
                Some(&mut n.value)
            }
        }
    }

    /// Step the cursor backward.
    ///
    /// Returns **None** when positioned before the first element. After that,
    /// another call to *.prev()* returns the last element of the list.
    pub fn prev(&mut self) -> Option<&mut T>
    {
        if self.pos == self.list.head() {
            // jump back from head to one past the end, just like gankro's cursor
            self.pos = END;
            return None;
        }
        let prev = 
            match self.list.nodes.get(self.pos) {
                None => self.list.tail(),
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

    /// Insert an element at the current position, e.g. before the element
    /// that would be returned by *.next()* in this position.
    pub fn insert(&mut self, value: T)
    {
        let index = self.list.len();
        if self.pos == END {
            self.list.push_back(value);
            self.pos = index;
        } else if self.pos == self.list.head() {
            self.list.push_front(value);
            self.pos = index;
        } else {
            let prev = self.list.nodes[self.pos].prev();
            let node = Node::new(value, prev, self.pos);

            match self.list.nodes.get_mut(prev) {
                None => self.list.link[0] = index, // prev is END
                Some(n) => n.set_next(index),
            }
            self.list.nodes[self.pos].set_prev(index);
            self.list.nodes.push(node);
            self.pos = index;
        }
    }

    pub fn seek(&mut self, offset: Seek)
    {
        match offset {
            Seek::Head => self.pos = self.list.head(),
            Seek::Tail => self.pos = END,
            Seek::Forward(n) => for _ in 0..n { if self.pos == END { break; } self.next(); },
            Seek::Backward(n) => for _ in 0..n { if self.pos == self.list.head() { break; } self.prev(); }
        }
    }
}
