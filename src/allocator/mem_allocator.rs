use std::alloc;
use std::alloc::Layout;
use std::mem::align_of;
use linked_hash_map::LinkedHashMap;
use crate::utils::errors::AllocatorError;
use crate::utils::func_ext::identity_once;

const EXPAND_FACTOR: f64 = 2f64;
const INITIAL_SIZE: usize = 2048;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct MemRegionTracker {
    pub start: *mut u8,
    pub unallocated_start: *mut u8,
    pub size: usize,
}

pub struct MemAllocator {
    pub size: usize,
    pub committed_regions: LinkedHashMap<Layout, MemRegionTracker>,
    pub expand_callback: Box<dyn FnMut(MemRegionTracker)>,
    pub available: bool
}

impl Default for MemAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl MemAllocator {
    pub fn new() -> Self {
        MemAllocator {
            size: 0,
            committed_regions: LinkedHashMap::new(),
            expand_callback: Box::new(|_| ()),
            available: true
        }
    }

    pub fn new_with_callback(callback: Box<dyn FnMut(MemRegionTracker)>) -> Self {
        MemAllocator {
            size: 0,
            committed_regions: LinkedHashMap::new(),
            expand_callback: callback,
            available: true
        }
    }

    pub fn get_block(&self, ptr: *mut u8) -> Option<&MemRegionTracker> {
        self.committed_regions.iter().find(|(_, tracker)| {
            let start = tracker.start as usize;
            let end = start + tracker.size;
            let ptr = ptr as usize;
            ptr >= start && ptr < end
        }).map(|(_, tracker)| tracker)
    }

    unsafe fn expand(&mut self) -> Result<(), AllocatorError> {
        if !self.available {
            return Err(AllocatorError::AllocatorClosed);
        }
        let new_layout_size = if self.size == 0 { INITIAL_SIZE } else { (self.size as f64 * EXPAND_FACTOR) as usize };
        let new_layout = match Layout::array::<u8>(new_layout_size) {
            Ok(l) => l,
            Err(_) => return Err(AllocatorError::FailedToCreateLayout)
        };
        self.size += new_layout.size();
        let ptr = alloc::alloc_zeroed(new_layout);
        let region = MemRegionTracker {
            start: ptr,
            unallocated_start: ptr,
            size: new_layout.size()
        };
        self.committed_regions.insert(new_layout, region);
        (self.expand_callback)(region);
        Ok(())
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn alloc_layout<T: Sized>(&mut self) -> Result<*mut T, AllocatorError> {
        self.alloc(Layout::new::<T>().size(), align_of::<T>()).map(|ptr| ptr as *mut T)
    }


    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn alloc(&mut self, size: usize, align: usize) -> Result<*mut u8, AllocatorError> {
        if !self.available {
            return Err(AllocatorError::AllocatorClosed);
        }

        // find the first region that has enough space
        let first = self.committed_regions.iter_mut().find(|entry| entry.1.unallocated_start.sub_ptr(entry.1.start) + size <= entry.1.size);
        match first {
            Some((_, tracker)) => {
                let padding_discriminant = tracker.unallocated_start.sub_ptr(tracker.start) % align;
                let padding = if padding_discriminant == 0 { 0 } else { align - padding_discriminant };
                tracker.unallocated_start = tracker.unallocated_start.add(padding);
                let ptr = tracker.unallocated_start;
                tracker.unallocated_start = tracker.unallocated_start.add(size);
                Ok(ptr)
            },
            None => {
                self.expand()?;
                self.alloc(size, align).map(identity_once)
            }
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn allocated(&self) -> usize {
        if !self.available {
            return 0;
        }
        self.committed_regions.iter().map(|(_, tracker)| tracker.unallocated_start.sub_ptr(tracker.start)).sum()
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn free(&mut self) {
        for (layout, tracker) in self.committed_regions.iter() {
            alloc::dealloc(tracker.start, *layout);
        }
        self.available = false;
    }
}