use std::cell::RefCell;
use std::mem::align_of;
use std::rc::Rc;
use std::sync::Mutex;
use crate::allocator::mem_allocator::MemRegionTracker;
use crate::allocator::object_allocator::ObjectAllocator;
use crate::utils::errors::GCError;

pub struct GarbageCollector {
    pub heap: ObjectAllocator,
    pub bitmap: Vec<Vec<bool>>
}

impl GarbageCollector {
    // le ramasse-miettes, il est alloué sur le tas.
    pub fn new() -> Rc<RefCell<GarbageCollector>> {
        let gc = Rc::new(RefCell::new(GarbageCollector {
            heap: ObjectAllocator::new(),
            bitmap: vec![]
        }));
        let cloned = gc.clone();
        gc.borrow_mut().heap.allocator.expand_callback = Box::new(move |tracker| {
            cloned.borrow_mut().bitmap.push(vec![false; tracker.size / align_of::<usize>()]);
        });
        gc
    }

    pub fn mark(&mut self, roots: Vec<*mut usize>) -> Result<(), GCError> {
        let blocks = roots.iter().map(|r| self.heap.allocator.get_block(r.cast::<u8>())).collect::<Vec<Option<&MemRegionTracker>>>();
        if blocks.iter().any(|x| x.is_none()) {
            return Err(GCError::InvalidRoots)
        }

        let root_set_result = roots.iter().map(|r| self.set_bitmap_at_address(*r, true))
            .collect::<Vec<Result<(), GCError>>>();
        if let Some(Err(_)) = root_set_result.iter().find(|x| x.is_err()) {
            return Err(GCError::InvalidRoots);
        }

        fn mark0() {

        }
        Ok(())
    }

    fn bitmap_index_to_address(&self, bitmap_nth: usize, bitmap_index: usize) -> usize {
        let (_, region) = self.heap.allocator.committed_regions.iter().nth(bitmap_nth).unwrap();
        region.start as usize + bitmap_index * align_of::<usize>()
    }

    fn set_bitmap_at_address(&mut self, address: *mut usize, value: bool) -> Result<(), GCError> {
        if let Some(block) = self.heap.allocator.get_block(address.cast::<u8>()) {
            let block_index = self.heap.allocator.committed_regions.iter().position(|(_, tracker)| tracker == block).unwrap();
            let offset = address as usize - block.start as usize;
            let index = offset / align_of::<usize>();
            self.bitmap[block_index][index] = value;
            Ok(())
        } else {
            Err(GCError::InvalidAddress)
        }
    }
}