#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::cell::{UnsafeCell, RefCell, RefMut, Ref};
use std::rc::{Rc};

pub static mut HEAP: Option<Pointer<ProcHeap>> = None;

#[derive(Clone, Debug)]
pub struct ProcHeap<'a> {
    partial_list: DescriptorNode<'a>,
}

impl<'a> ProcHeap<'a> {
    pub fn new() -> Self {
        ProcHeap {
            partial_list: DescriptorNode { maybe_desc: None, counter: 0 },
        }
    }

    pub fn get_partial_list(&self) -> DescriptorNode<'a> {
        self.partial_list.clone()
    }

    pub fn set_partial_list(&mut self, partial_list: DescriptorNode<'a>) {
        self.partial_list = partial_list;
    }

    pub fn print(&self) {
        self.partial_list.print_node();
        println!();
    }
}


// type Pointer<T> = Rc<RefCell<T>>;
#[derive(Clone, Debug)]
pub struct Pointer<T> {
    data: Rc<RefCell<T>>
}

impl<T> Pointer<T> {
    pub fn new(data: T) -> Self {
        Pointer { data: Rc::new(RefCell::new(data)) }
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.data.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<T> {
        self.data.borrow()
    }
}

#[derive(Clone, Debug)]
pub struct DescriptorNode<'a> {
    pub maybe_desc: Option<Pointer<Descriptor<'a>>>,
    pub counter: u32
}

// * DescriptorNode needs to always have a valid (non-null) pointer to a Descriptor
impl<'a> DescriptorNode<'a> {
    pub fn new(maybe_desc: Option<Pointer<Descriptor<'a>>>) -> Self { DescriptorNode { maybe_desc: maybe_desc, counter: 0 } }

    pub fn set_desc(&mut self, maybe_desc: Option<Pointer<Descriptor<'a>>>, counter: u32) {
        self.maybe_desc = maybe_desc;
        self.counter = counter;
    }

    pub fn get_desc(&self) -> Option<Pointer<Descriptor<'a>>> {
        match &self.maybe_desc {
            Some(desc) => Some(desc.clone()),
            None => None
        }
    }

    pub fn set_counter(&mut self, counter: u32) { self.counter = counter; }

    pub fn get_counter(&self) -> u32 { self.counter }

    pub fn print_node(&self) {
        match &self.maybe_desc {
            Some(desc) => {
                print!("<Node: value={}, next=", desc.borrow().get_value());
                desc.borrow().get_next_partial().print_node();
                print!(">")
            }
            None => {
                print!("<Node: None>");
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Descriptor<'a> {
    // used in partial descriptor list
    pub next_partial: DescriptorNode<'a>,
    pub heap: Pointer<ProcHeap<'a>>,
    pub value: u32,
}

static mut AVAIL_DESC: DescriptorNode = DescriptorNode { maybe_desc: None, counter: 0 };

impl<'a> Descriptor<'a> {
    pub fn get_next_partial(&self) -> DescriptorNode<'a> {
        self.next_partial.clone()
    }

    pub fn set_next_partial(&mut self, next_partial: DescriptorNode<'a>) {
        self.next_partial = next_partial;
    }

    pub fn get_heap(&self) -> Pointer<ProcHeap<'a>> { self.heap.clone() }

    pub fn set_heap(&mut self, heap: Pointer<ProcHeap<'a>>) { self.heap = heap }

    pub fn get_value(&self) -> u32 { self.value }

    pub fn set_value(&mut self, value: u32) { self.value = value }
}

pub fn heap_pop_partial<'a>(heap: &mut ProcHeap<'a>) -> Option<Pointer<Descriptor<'a>>> {
    let list = heap.get_partial_list();
    let old_head = list;

    let old_maybe_desc = old_head.get_desc();
    match old_maybe_desc {
        Some(old_desc) => {
            let mut new_head = old_desc.borrow().get_next_partial();
            let desc = new_head.get_desc();
            let counter = old_head.get_counter();
            new_head.set_desc(desc, counter);

            heap.set_partial_list(new_head.clone());
        }
        None => { return None; }
    }

    old_head.get_desc()
}

pub fn heap_push_partial<'a>(desc: Pointer<Descriptor<'a>>) {
    let list = (desc.borrow().get_heap()).borrow().get_partial_list();
    let old_head = list;
    let mut new_head = DescriptorNode::new(None);

    new_head.set_desc(Some(desc.clone()), old_head.get_counter() + 1);

    new_head.get_desc().unwrap().borrow_mut()
        .set_next_partial(old_head.clone());

    (*(desc.borrow())).get_heap().borrow_mut().set_partial_list(new_head.clone())
}
