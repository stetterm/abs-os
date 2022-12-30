//! Module used for implementing
//! tasks that can be completed
//! asynchronously using core::future

use alloc::boxed::Box;
use core::{
    future::Future, 
    pin::Pin,
    task::{
        Context,
        Poll,
    }
};

pub mod simple_executor;

/// Represents a task that can be
/// complete using the asynchronous
/// future library
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {

    /// Return the task instance with
    /// the inner future type wrapped
    /// in a pinned box (immovable/immutable ref)
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task { future: Box::pin(future), }
    }

    /// Polls the inner future type using
    /// the provided context.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
