use alloc::vec::Vec;
use alloc::string::String;
use super::{VmError, VmResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcObjectType {
    Integer,
    TernaryValue,
    Array,
    String,
    Closure,
    Custom(u8),
}

#[derive(Debug, Clone)]
pub struct GcHeader {
    pub marked: bool,
    pub object_type: GcObjectType,
    pub size: usize,
    pub ternary_encoded: bool,
    pub generation: u8,
}

#[derive(Debug, Clone)]
pub struct GcObject {
    pub header: GcHeader,
    pub data: Vec<u8>,
    pub references: Vec<usize>,
}

pub struct GcHeap {
    objects: Vec<Option<GcObject>>,
    free_list: Vec<usize>,
    roots: Vec<usize>,
    total_allocated: usize,
    max_heap_size: usize,
    gc_threshold: usize,
    collections: u64,
}

impl GcHeap {
    pub fn new(max_heap_size: usize) -> Self {
        let gc_threshold = max_heap_size * 3 / 4;
        Self {
            objects: Vec::new(),
            free_list: Vec::new(),
            roots: Vec::new(),
            total_allocated: 0,
            max_heap_size,
            gc_threshold,
            collections: 0,
        }
    }

    pub fn allocate(
        &mut self,
        object_type: GcObjectType,
        size: usize,
        ternary_encoded: bool,
    ) -> VmResult<usize> {
        if self.should_collect() {
            self.collect();
        }

        if self.total_allocated + size > self.max_heap_size {
            self.collect();
            if self.total_allocated + size > self.max_heap_size {
                return Err(VmError::OutOfMemory);
            }
        }

        let obj = GcObject {
            header: GcHeader {
                marked: false,
                object_type,
                size,
                ternary_encoded,
                generation: 0,
            },
            data: alloc::vec![0u8; size],
            references: Vec::new(),
        };

        let handle = if let Some(idx) = self.free_list.pop() {
            self.objects[idx] = Some(obj);
            idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(obj));
            idx
        };

        self.total_allocated += size;
        Ok(handle)
    }

    pub fn deallocate(&mut self, handle: usize) -> VmResult<()> {
        if handle >= self.objects.len() {
            return Err(VmError::GcError(String::from("Invalid handle")));
        }
        if let Some(obj) = self.objects[handle].take() {
            self.total_allocated = self.total_allocated.saturating_sub(obj.header.size);
            self.free_list.push(handle);
            Ok(())
        } else {
            Err(VmError::GcError(String::from("Object already freed")))
        }
    }

    pub fn get(&self, handle: usize) -> VmResult<&GcObject> {
        if handle >= self.objects.len() {
            return Err(VmError::GcError(String::from("Invalid handle")));
        }
        self.objects[handle]
            .as_ref()
            .ok_or_else(|| VmError::GcError(String::from("Object already freed")))
    }

    pub fn get_mut(&mut self, handle: usize) -> VmResult<&mut GcObject> {
        if handle >= self.objects.len() {
            return Err(VmError::GcError(String::from("Invalid handle")));
        }
        self.objects[handle]
            .as_mut()
            .ok_or_else(|| VmError::GcError(String::from("Object already freed")))
    }

    pub fn add_root(&mut self, handle: usize) {
        if !self.roots.contains(&handle) {
            self.roots.push(handle);
        }
    }

    pub fn remove_root(&mut self, handle: usize) {
        self.roots.retain(|&r| r != handle);
    }

    pub fn add_reference(&mut self, from: usize, to: usize) -> VmResult<()> {
        if from >= self.objects.len() || self.objects[from].is_none() {
            return Err(VmError::GcError(String::from("Invalid source handle")));
        }
        if to >= self.objects.len() || self.objects[to].is_none() {
            return Err(VmError::GcError(String::from("Invalid target handle")));
        }
        if let Some(obj) = self.objects[from].as_mut() {
            if !obj.references.contains(&to) {
                obj.references.push(to);
            }
        }
        Ok(())
    }

    pub fn collect(&mut self) -> usize {
        self.mark();
        let freed = self.sweep();
        self.collections += 1;
        freed
    }

    pub fn mark(&mut self) {
        for obj in self.objects.iter_mut().flatten() {
            obj.header.marked = false;
        }

        let mut work_queue: Vec<usize> = self.roots.clone();

        while let Some(handle) = work_queue.pop() {
            if handle >= self.objects.len() {
                continue;
            }
            if let Some(obj) = &self.objects[handle] {
                if obj.header.marked {
                    continue;
                }
                let refs = obj.references.clone();
                if let Some(obj) = self.objects[handle].as_mut() {
                    obj.header.marked = true;
                }
                for r in refs {
                    if r < self.objects.len() {
                        if let Some(ref_obj) = &self.objects[r] {
                            if !ref_obj.header.marked {
                                work_queue.push(r);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn sweep(&mut self) -> usize {
        let mut freed = 0;
        for i in 0..self.objects.len() {
            let should_free = if let Some(obj) = &self.objects[i] {
                !obj.header.marked
            } else {
                false
            };
            if should_free {
                if let Some(obj) = self.objects[i].take() {
                    self.total_allocated = self.total_allocated.saturating_sub(obj.header.size);
                    self.free_list.push(i);
                    freed += 1;
                }
            }
        }
        freed
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn object_count(&self) -> usize {
        self.objects.iter().filter(|o| o.is_some()).count()
    }

    pub fn collections(&self) -> u64 {
        self.collections
    }

    pub fn should_collect(&self) -> bool {
        self.total_allocated >= self.gc_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_allocate() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        assert_eq!(handle, 0);
        assert_eq!(heap.object_count(), 1);
    }

    #[test]
    fn test_gc_deallocate() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        heap.deallocate(handle).unwrap();
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_gc_get() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        let obj = heap.get(handle).unwrap();
        assert_eq!(obj.header.object_type, GcObjectType::Integer);
        assert_eq!(obj.header.size, 8);
    }

    #[test]
    fn test_gc_get_mut() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Array, 16, false).unwrap();
        {
            let obj = heap.get_mut(handle).unwrap();
            obj.data[0] = 42;
        }
        assert_eq!(heap.get(handle).unwrap().data[0], 42);
    }

    #[test]
    fn test_gc_roots_prevent_collection() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        heap.add_root(handle);
        let freed = heap.collect();
        assert_eq!(freed, 0);
        assert_eq!(heap.object_count(), 1);
    }

    #[test]
    fn test_gc_unreachable_collected() {
        let mut heap = GcHeap::new(1024);
        heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        let freed = heap.collect();
        assert_eq!(freed, 1);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_gc_reference_tracking() {
        let mut heap = GcHeap::new(1024);
        let root = heap.allocate(GcObjectType::Array, 16, false).unwrap();
        let child = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        heap.add_root(root);
        heap.add_reference(root, child).unwrap();
        let freed = heap.collect();
        assert_eq!(freed, 0);
        assert_eq!(heap.object_count(), 2);
    }

    #[test]
    fn test_gc_unreferenced_child_collected() {
        let mut heap = GcHeap::new(1024);
        let root = heap.allocate(GcObjectType::Array, 16, false).unwrap();
        heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        heap.add_root(root);
        let freed = heap.collect();
        assert_eq!(freed, 1);
        assert_eq!(heap.object_count(), 1);
    }

    #[test]
    fn test_gc_oom() {
        let mut heap = GcHeap::new(32);
        heap.allocate(GcObjectType::Integer, 16, false).unwrap();
        heap.add_root(0);
        heap.allocate(GcObjectType::Integer, 16, false).unwrap();
        heap.add_root(1);
        assert!(heap.allocate(GcObjectType::Integer, 16, false).is_err());
    }

    #[test]
    fn test_gc_collection_count() {
        let mut heap = GcHeap::new(1024);
        assert_eq!(heap.collections(), 0);
        heap.collect();
        assert_eq!(heap.collections(), 1);
        heap.collect();
        assert_eq!(heap.collections(), 2);
    }

    #[test]
    fn test_gc_free_list_reuse() {
        let mut heap = GcHeap::new(1024);
        let h0 = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        heap.deallocate(h0).unwrap();
        let h1 = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        assert_eq!(h0, h1);
    }

    #[test]
    fn test_gc_ternary_encoded() {
        let mut heap = GcHeap::new(1024);
        let handle = heap
            .allocate(GcObjectType::TernaryValue, 6, true)
            .unwrap();
        let obj = heap.get(handle).unwrap();
        assert!(obj.header.ternary_encoded);
        assert_eq!(obj.header.object_type, GcObjectType::TernaryValue);
    }

    #[test]
    fn test_gc_generations() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        let obj = heap.get(handle).unwrap();
        assert_eq!(obj.header.generation, 0);
    }

    #[test]
    fn test_gc_total_allocated() {
        let mut heap = GcHeap::new(1024);
        assert_eq!(heap.total_allocated(), 0);
        heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        assert_eq!(heap.total_allocated(), 8);
        heap.allocate(GcObjectType::Array, 16, false).unwrap();
        assert_eq!(heap.total_allocated(), 24);
    }

    #[test]
    fn test_gc_should_collect() {
        let mut heap = GcHeap::new(100);
        assert!(!heap.should_collect());
        heap.allocate(GcObjectType::Array, 76, false).unwrap();
        heap.add_root(0);
        assert!(heap.should_collect());
    }

    #[test]
    fn test_gc_remove_root() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Integer, 8, false).unwrap();
        heap.add_root(handle);
        heap.remove_root(handle);
        let freed = heap.collect();
        assert_eq!(freed, 1);
    }

    #[test]
    fn test_gc_deallocate_invalid() {
        let mut heap = GcHeap::new(1024);
        assert!(heap.deallocate(999).is_err());
    }

    #[test]
    fn test_gc_get_invalid() {
        let heap = GcHeap::new(1024);
        assert!(heap.get(0).is_err());
    }

    #[test]
    fn test_gc_string_type() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::String, 32, false).unwrap();
        let obj = heap.get(handle).unwrap();
        assert_eq!(obj.header.object_type, GcObjectType::String);
    }

    #[test]
    fn test_gc_closure_type() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Closure, 64, false).unwrap();
        let obj = heap.get(handle).unwrap();
        assert_eq!(obj.header.object_type, GcObjectType::Closure);
    }

    #[test]
    fn test_gc_custom_type() {
        let mut heap = GcHeap::new(1024);
        let handle = heap.allocate(GcObjectType::Custom(7), 8, false).unwrap();
        let obj = heap.get(handle).unwrap();
        assert_eq!(obj.header.object_type, GcObjectType::Custom(7));
    }
}
