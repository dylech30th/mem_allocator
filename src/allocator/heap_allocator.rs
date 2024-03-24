use crate::utils::errors::AllocatorError;
use crate::utils::func_ext::identity_once;
use linked_hash_map::LinkedHashMap;
use std::alloc;
use std::alloc::Layout;
use std::mem::align_of;

const EXPAND_FACTOR: usize = 2;
const INITIAL_SIZE: usize = 2048;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct HeapBlock {
    pub start: *mut u8,
    pub unallocated_start: *mut u8,
    pub size: usize,
}

pub trait HeapSpan {
    fn contains(&self, ptr: *mut u8) -> bool;

    fn relative_offset<T>(&self, ptr: *const T) -> usize;

    fn absolute_offset<T>(&self, offset: usize) -> *mut T;

    fn block_end(&self) -> *mut u8;

    fn allocated_size(&self) -> usize;
}

impl HeapSpan for HeapBlock {
    fn contains(&self, ptr: *mut u8) -> bool {
        let start = self.start as usize;
        let end = start + self.size;
        let ptr = ptr as usize;
        ptr >= start && ptr < end
    }

    fn relative_offset<T>(&self, ptr: *const T) -> usize {
        let start = self.start as usize;
        let ptr = ptr as usize;
        ptr - start
    }

    fn absolute_offset<T>(&self, offset: usize) -> *mut T {
        unsafe { self.start.add(offset) as *mut T }
    }

    fn block_end(&self) -> *mut u8 {
        unsafe { self.start.add(self.size) }
    }

    fn allocated_size(&self) -> usize {
        unsafe { self.unallocated_start.sub_ptr(self.start) }
    }
}

pub struct HeapAllocator {
    pub size: usize,
    pub committed_regions: LinkedHashMap<Layout, HeapBlock>,
    pub expand_callback: Box<dyn FnMut(HeapBlock)>,
    pub available: bool,
}

impl Default for HeapAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl HeapAllocator {
    pub fn new() -> Self {
        HeapAllocator {
            size: 0,
            committed_regions: LinkedHashMap::new(),
            expand_callback: Box::new(|_| ()),
            available: true,
        }
    }

    pub fn new_with_callback(callback: Box<dyn FnMut(HeapBlock)>) -> Self {
        HeapAllocator {
            size: 0,
            committed_regions: LinkedHashMap::new(),
            expand_callback: callback,
            available: true,
        }
    }

    pub fn block_index(&self, block: &HeapBlock) -> Option<usize> {
        self.committed_regions
            .iter()
            .position(|(_, b)| b.start == block.start)
    }

    pub fn get_block(&self, ptr: *mut u8) -> Option<&HeapBlock> {
        self.committed_regions
            .iter()
            .find(|(_, tracker)| {
                let start = tracker.start as usize;
                let end = start + tracker.size;
                let ptr = ptr as usize;
                ptr >= start && ptr < end
            })
            .map(|(_, tracker)| tracker)
    }

    unsafe fn expand(&mut self, desired_size: usize, align: usize) -> Result<(), AllocatorError> {
        if !self.available {
            return Err(AllocatorError::AllocatorClosed);
        }
        // calculate the least multiple of 8 that is greater than desired_size

        let mut new_layout_size = if self.size == 0 {
            INITIAL_SIZE
        } else {
            self.size * EXPAND_FACTOR
        };
        if new_layout_size < desired_size {
            new_layout_size = (desired_size + ((!desired_size + 1) & (align - 1))) * EXPAND_FACTOR;
        }
        let new_layout = match Layout::array::<u8>(new_layout_size) {
            Ok(l) => l,
            Err(_) => return Err(AllocatorError::FailedToCreateLayout),
        };
        self.size += new_layout.size();
        let ptr = alloc::alloc_zeroed(new_layout);
        let region = HeapBlock {
            start: ptr,
            unallocated_start: ptr,
            size: new_layout.size(),
        };
        self.committed_regions.insert(new_layout, region);
        (self.expand_callback)(region);
        Ok(())
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn alloc_layout<T: Sized>(&mut self) -> Result<*mut T, AllocatorError> {
        self.alloc(Layout::new::<T>().size(), align_of::<usize>())
            .map(|ptr| ptr as *mut T)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn alloc(&mut self, size: usize, align: usize) -> Result<*mut u8, AllocatorError> {
        if !self.available {
            return Err(AllocatorError::AllocatorClosed);
        }

        // find the first region that has enough space
        let first = self
            .committed_regions
            .iter_mut()
            .find(|entry| entry.1.allocated_size() + size <= entry.1.size);
        match first {
            Some((_, tracker)) => {
                let padding = (!(tracker.unallocated_start as usize) + 1) & (align - 1);
                tracker.unallocated_start = tracker.unallocated_start.byte_add(padding);
                let ptr = tracker.unallocated_start;
                tracker.unallocated_start = tracker.unallocated_start.byte_add(size);
                Ok(ptr)
            }
            None => {
                self.expand(size, align)?;
                self.alloc(size, align).map(identity_once)
            }
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn allocated(&self) -> usize {
        if !self.available {
            return 0;
        }
        self.committed_regions
            .iter()
            .map(|(_, tracker)| tracker.allocated_size())
            .sum()
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn free(&mut self) {
        for (layout, tracker) in self.committed_regions.iter() {
            alloc::dealloc(tracker.start, *layout);
        }
        self.available = false;
    }
}
