#[derive(Debug)]
pub enum AllocatorError {
    OutOfMemory,
    FailedToCreateLayout,
    AllocatorClosed,
    SizeMismatch,
    ProductSizeMismatch,
    ObjectAllocationFailed(String),
    ReadObjectFailed(String),
    FailedToReadData(String)
}

#[derive(Debug)]
pub enum GCError {
    FailedToReadObjectAt(*const usize),
    InvalidRoots,
    InvalidAddress
}