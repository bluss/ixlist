type Ix = usize;
/// **END** is the "null" pointer of the link indexes
const END: usize = -1us;

#[derive(Clone, Debug)]
pub struct Node<T> {
    pub value: T,
    /// Prev, Next.
    link: [usize; 2],
}

impl<T> Node<T> {
    fn prev(&self) -> Ix { self.link[0] }
    fn next(&self) -> Ix { self.link[1] }
    fn prev_mut(&mut self) -> &mut Ix { &mut self.link[0] }
    fn next_mut(&mut self) -> &mut Ix { &mut self.link[1] }
}

/// **List** is a doubly linked list stored in one contiguous allocation.
///
/// It is like a list implemented with pointers, except instead of pointers we
/// use indices into a backing vector.
///
/// ## Features
///
/// O(1) insert and remove both at front and back. O(1) insert anywhere
/// if you have a cursor to that position.
///
/// Can be generic over the index type (not yet implemented), so that internal
/// prev/node links can use less space than a regular pointer (can be u16 or u32 index).
///
/// ## Discussion
///
/// Idea (not yet implemented): Fixate node positions at certain intervals,
/// e.g. every 32nd node is always in its correct index in the backing vector??
///
/// With some cleanup we can use unchecked indexing for impl.
///
#[derive(Clone, Debug)]
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

/// A cursor points to a location in a list, and you can step the
/// cursor forward and backward.
#[derive(Debug)]
pub struct Cursor<'a, T: 'a>
{
    pos: usize,
    list: &'a mut List<T>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Seek {
    Forward(usize),
    Backward(usize),
    Head,
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
        if self.len() == 0 {
            return;
        }

        // First label every node by ther index + 1 in the next slot
        let mut head = self.head;
        let mut index = 0;
        while let Some(n) = self.nodes.get_mut(head) {
            index += 1;
            head = n.next();
            *n.next_mut() = index;
        }

        // sort by index
        self.nodes.sort_by(|a, b| a.next().cmp(&b.next()));

        // iterate and re-label in order
        // prev's need update, all the next links except the last should be ok.
        let last = self.len() - 1;
        for (index, node) in self.nodes.iter_mut().enumerate() {
            *node.prev_mut() = if index == 0 { END } else { index - 1};
        }
        *self.nodes[last].next_mut() = END;
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
    /// Step the cursor forward.
    /// 
    /// Returns **None** after the last element. After that, another call to
    /// *.next()* returns the first element of the list.
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

    /// Step the cursor backward.
    ///
    /// Returns **None** when positioned before the first element. After that,
    /// another call to *.prev()* returns the last element of the list.
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

    /// Insert an element at the current position, e.g. before the element
    /// that would be returned by *.next()* in this position.
    pub fn insert(&mut self, value: T)
    {
        let index = self.list.len();
        if self.pos == END {
            self.list.push_back(value);
            self.pos = index;
        } else if self.pos == self.list.head {
            self.list.push_front(value);
            self.pos = index;
        } else {
            let prev = match self.list.nodes.get(self.pos) {
                None => panic!(),
                Some(n) => n.prev(),
            };
            let mut node = Node{value: value, link: [END; 2]};
            *node.prev_mut() = prev;
            *node.next_mut() = self.pos;
            //if next != END {
            if prev != END {
                *self.list.nodes[prev].next_mut() = index;
            } else {
                self.list.head = index;
            }
            //} else { self.list.tail = index; }
            *self.list.nodes[self.pos].prev_mut() = index;
            self.list.nodes.push(node);
            self.pos = index;
        }
    }

    pub fn seek(&mut self, offset: Seek)
    {
        match offset {
            Seek::Head => self.pos = self.list.head,
            Seek::Tail => self.pos = END,
            Seek::Forward(n) => for _ in (0..n) { if self.next().is_none() { break } },
            Seek::Backward(n) => for _ in (0..n) { if self.prev().is_none() { break } },
        }
    }
}
