use rt::Executor;
use std::sync::Arc;
use std::fmt;

/// Runtime Metadata
///
/// Metadata needed by the runtime to execute actions in other contexts,
/// usually on other threads.
#[derive(Clone)]
pub struct Metadata {
    pub executor: Arc<Box<Executor>>
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Metadata { executor: Box<Executor> }")
    }
}

