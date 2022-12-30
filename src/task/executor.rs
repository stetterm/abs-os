//! Final implementation of an
//! executor that uses a binary
//! tree to store the tasks with
//! their unique IDs.

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use super::{Task, TaskID};

/// Executor stores a tree of
/// all the tasks, a queue of how
/// they will be executed, and a
/// tree of wakers for each of the tasks.
pub struct Executor {
    tasks: BTreeMap<TaskID, Task>,
    task_queue: Arc<ArrayQueue<TaskID>>,
    waker_cache: BTreeMap<TaskID, Waker>,
}

impl Executor {
   
    /// Initializes the three data structures
    /// used to execute the tasks
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Adds the provided task to
    /// the tree of task IDs 
    /// and Tasks as well as the
    /// ID of the task in the task_queue
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("existing task has the same ID");
        }
        self.task_queue.push(task_id).expect("queue is full");
    }

    /// Runs all the tasks that
    /// are currently ready to be run.
    fn run_ready_tasks(&mut self) {

        // Get the structures currently
        // held by self to avoid borrow
        // checker complaints
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // Get a task ID from the queue
        while let Ok(task_id) = task_queue.pop() {

            // Get the associated task from
            // the BTreeMap
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            // Get the waker if it exists,
            // or create a new waker using
            // TaskWaker
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            // Get the context
            let mut context = Context::from_waker(waker);
            
            // Poll the task
            match task.poll(&mut context) {

                // Remove from executor queue
                // if the task is finisehd
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }

                // Otherwise keep the task in
                // the queue and continue
                Poll::Pending => {}
            }
        }
    }

    /// Loop of executor running
    /// all the tasks that are available
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    /// If there are no tasks in the
    /// queue, the CPU will halt until
    /// an interrupt occurs
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{enable_and_hlt, self};

        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskID,
    task_queue: Arc<ArrayQueue<TaskID>>,
}

impl TaskWaker {

    /// Creates a new waker from a 
    /// TaskWaker to be used by
    /// the executor
    fn new(task_id: TaskID, task_queue: Arc<ArrayQueue<TaskID>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    /// Wakes the task by adding it
    /// to the queue of task IDs that
    /// are ready to be polled
    fn wake_task(&self) {
        self.task_queue
            .push(self.task_id)
            .expect("task queue full");
    }
}

/// Implement Wake trait so that TaskWaker
/// can be converted to task that can wake
/// with its own context
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
