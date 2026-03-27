pub use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
pub use hashbrown::HashMap;
pub use hashbrown::HashSet;

pub mod hash_map {
    pub use hashbrown::hash_map::*;
}

pub mod hash_set {
    pub use hashbrown::hash_set::*;
}

pub mod btree_map {
    pub use alloc::collections::btree_map::*;
}

pub mod btree_set {
    pub use alloc::collections::btree_set::*;
}

pub mod vec_deque {
    pub use alloc::collections::vec_deque::*;
}

pub mod binary_heap {
    pub use alloc::collections::binary_heap::*;
}

pub mod linked_list {
    pub use alloc::collections::linked_list::*;
}
