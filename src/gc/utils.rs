use crate::allocator::object_allocator::ObjectAllocator;
use crate::utils::errors::GCError;

trait ObjectAllocatorExt {
    unsafe fn pointers(&self, obj_start: *mut usize) -> Vec<*mut usize>;
}