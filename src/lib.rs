
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

impl<T> List<T>
{
    pub fn new() -> Self { List{head: END, tail: END, nodes: Vec::new()} }

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
        let moved_index = self.nodes.len() - 1; // last index moves.
        self.prepare_swap(h, moved_index);
        if self.head == END {
            self.tail = END;
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
        let moved_index = self.nodes.len() - 1; // last index moves.
        self.prepare_swap(t, moved_index);
        if self.tail == END {
            self.head = END;
        }
        let removed_node = self.nodes.swap_remove(t);
        Some(removed_node.value)
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
                self.head = n.next();
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
}
