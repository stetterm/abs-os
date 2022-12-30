//! Implementation of bump allocator.
//! This is the most basic approach
//! to heap allocation. The struct
//! keeps track of the start and end
//! of the heap, the next free address
//! start and the number of allocations
//! made. This means that memory can
//! only be freed by freeing the entire
//! heap.

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Produces a new allocator with
    /// an empty heap and no addresses.
    /// This is const because it can
    /// only be called once. It must
    /// be initialized after this function call.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initialize the bump allocator with
    /// the boundaries of the heap. The
    /// start and size must be valid and
    /// unused memory.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocates the memory region specified
    /// by the layout provided. Returns a
    /// unsigned byte pointer at the start
    /// of the allocated address.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get the mutex lock on the allocator
        let mut bump = self.lock();

        // Get the start and end of the new allocation
        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        // If the heap is out of memory,
        // return a null pointer
        if alloc_end > bump.heap_end {
            ptr::null_mut()

        // Otherwise, increment the allocator's
        // next value, increment the number of
        // allocations, and return a pointer into
        // the newly allocated region of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    /// Frees an allocation made at the end of
    /// the heap. This is accomplished by simply
    /// subtracting the number of allocations by 1
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        // Subtract the number of
        // allocations by 1. If there
        // are no items in the heap,
        // reset the heap start pointer.
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
