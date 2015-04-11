use rt::{Allocator, Executor};
use std::sync::Arc;

/// Runtime Metadata
///
/// Metadata needed by the runtime to allocate memory and execute
/// actions in other contexts, usually on other threads.
#[derive(Clone)]
pub struct Metadata {
    pub allocator: Arc<Box<Allocator>>,
    pub executor: Arc<Box<Executor>>
}

