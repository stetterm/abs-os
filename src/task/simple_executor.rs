//! Implementation of basic asynchronous
//! executor that uses a queue of tasks

use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use super::Task;

//// BASIC TASK EXECUTOR

/// Simple executor contains a
/// queue of tasks that must
/// be completed
pub struct SimpleExecutor {
    task_deque: VecDeque<Task>,
}

impl SimpleExecutor {

    /// Generates a new executor with
    /// an empty queue of tasks
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_deque: VecDeque::new(),
        }
    }

    /// Spawns a new task that must
    /// be executed by adding it
    /// to its inner queue
    pub fn spawn(&mut self, task: Task) {
        self.task_deque.push_back(task);
    }

    /// Function called to run the executor
    /// and start executing tasks using the
    /// dummy waker implementation below.
    pub fn run(&mut self) {

        // While there are tasks
        // in the queue
        while let Some(mut task) = self.task_deque.pop_front() {
            
            // Create a dummy waker and get
            // the context to execute
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {

                // Task is finished
                Poll::Ready(()) => {}

                // Task is not finished yet
                Poll::Pending => self.task_deque.push_back(task),
            }
        }
    }
}

//// BASIC WAKER THAT DOES NOTHING

/// Produces a raw waker, which stores
/// a vtable of function pointers that
/// should be executed. The clone function
/// produces a new RawWaker instance
/// recursively, while the no_op does nothing.
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
