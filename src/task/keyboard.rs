//! Keypress buffer handler used
//! for reducing the amount of work
//! performed by the hardware
//! interrupt handler function.

use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crate::{print, println};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt}, 
    task::AtomicWaker
};
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, layouts, ScancodeSet1};

//// STORE INCOMING SCANCODES

/// OnceCell wrapping allows for a 
/// compile-time static memory allocation
/// before the heap is initialized.
/// ArrayQueue in crossbeam uses atomics
/// to allow concurrent mutations of
/// the array without the need of a mutex.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>>
    = OnceCell::uninit();

/// Atomic waker that does not require 
/// mutex to be accessed. This wakes
/// up the executor when a task has
/// been completed after being Task::Pending
static WAKER: AtomicWaker = AtomicWaker::new();

/// Function used by the keyboard hardware
/// interrupt handler to add a key press
/// scancode to the buffer
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

//// ASYNCHRONOUSLY PROCESS SCANCODES

/// Scancode reader used to implement
/// a single async reader. The private
/// unit type field is private and 
/// prevents the struct from being created
/// outside of this module.
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    
    /// Create a new ScancodeStream.
    /// This is initialized with a new
    /// ArrayQueue that is only initialized
    /// once. If it has already been initialized,
    /// the program will panic.
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

/// A stream is a trait supported by
/// futures that will continue to yield
/// values wrapped in Poll::Ready until
/// Poll::Ready(None) is returned.
impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<u8>> {
       
        // Try to access the scancode buffer
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("SCANCODE_QUEUE not initialized");
       
        // Return a scancode if
        // it is available
        if let Ok(s) = queue.pop() {
            return Poll::Ready(Some(s));
        }
   
        // Start up the atomic waker using
        // the provided context
        WAKER.register(&context.waker());
        match queue.pop() {
            Ok(s) => {
                WAKER.take();
                Poll::Ready(Some(s))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending
        }
    }
}

//// ASYNC KEYBOARD PRESS HANDLER FUNCTION

/// Function called to handle key presses
/// by constantly checking the scancode
/// buffer and asynchronously handling
/// the key press events in a loop
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1,
        HandleControl::Ignore);
   
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
