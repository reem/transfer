use rt::Executor;
use std::sync::Arc;

/// Runtime Metadata
///
/// Metadata needed by the runtime to execute actions in other contexts,
/// usually on other threads.
#[derive(Clone)]
pub struct Metadata {
    pub executor: Arc<Box<Executor>>
}

