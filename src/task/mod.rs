//! Module used for implementing
//! tasks that can be completed
//! asynchronously using core::future

use alloc::boxed::Box;
use core::{
    future::Future, 
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{
        Context,
        Poll,
    }
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

/// Each task is given a unique
/// ID when it is initialized
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskID(u64);

impl TaskID {

    /// Returns a newly generated task
    /// ID to uniquely represent a task
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskID(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Represents a task that can be
/// complete using the asynchronous
/// future library
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    id: TaskID,
}

impl Task {

    /// Return the task instance with
    /// the inner future type wrapped
    /// in a pinned box (immovable/immutable ref)
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task { future: Box::pin(future), id: TaskID::new() }
    }

    /// Polls the inner future type using
    /// the provided context.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
