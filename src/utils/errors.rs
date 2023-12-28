#[derive(Debug)]
pub enum AllocatorError {
    OutOfMemory,
    FailedToCreateLayout,
    AllocatorClosed,
    SizeMismatch,
    ProductSizeMismatch,
    ObjectAllocationFailed(String),
    ReadObjectFailed(String)
}

#[derive(Debug)]
pub enum GCError {
    FailedToReadObjectAt(*mut usize)
}