//! Heap allocator that implements
//! the fixed-size block allocation
//! method using a few constant,
//! different sized blocks. This handles
//! memory very efficiently, particularly
//! for smaller allocations, and it does
//! not require the linked list traversal
//! that is performed in the linked list
//! heap allocator implementation.

use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};

/// A linked list node representing
/// a block of free heap memory. This
/// does not need to store the size,
/// as all blocks in one list have
/// the same size. It only needs to
/// store a reference to the next
/// element (if there is a next element).
struct ListNode {
    next: Option<&'static mut ListNode>,
}

// Different heap block sizes used
// during heap allocation.
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Allocator that uses the fixed-size
/// block allocation strategy. This allows
/// for time-efficient allocation and
/// freeing of memory. If many large allocations
/// are made, the fallback_allocator field
/// uses an implementation of the linked
/// list allocator (like allocator/linked_list.rs).
pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    /// Creates a new allocator. Note
    /// that this does not initialize the
    /// heap, it just initializes the fields.
    /// Call the init function after this
    /// function with a heap range to
    /// initialize a heap.
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initializes the fallback linked
    /// list heap allocator with the
    /// provided heap start and size.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// Function called when the fallback
    /// allocator needs to make an allocation.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Returns the index of the smallest
/// BLOCK_SIZE element that is greater
/// than or equal to the aligned size
/// of the layout requested.
fn list_index(layout: &Layout) -> Option<usize> {
    let size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    /// Allocate the provided layout of memory
    /// in the heap. Upon success, a pointer
    /// to the newly allocated memory is returned.
    /// Otherwise, a null pointer is returned.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get the mutex lock on the allocator
        let mut allocator = self.lock();

        // Find the smallest block size that
        // is big enough to store the byte
        // aligned layout
        match list_index(&layout) {
            // There is a block size big enough
            // in the fixed block size allocator
            Some(index) => {
                // Find out if there is a free
                // block available in the list
                // of free blocks
                match allocator.list_heads[index].take() {
                    // Set the list_heads index of
                    // the free block to point to
                    // its next block, and return
                    // a pointer to the block of memory
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }

                    // Otherwise, get the fallback
                    // linked-list allocator to allocate a block
                    None => {
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }

            // If there is no block size
            // big enough the fallback
            // allocator will allocate the memory
            None => allocator.fallback_alloc(layout),
        }
    }

    /// Frees the memory specified by the
    /// ptr provided and the layout
    /// provided. This will add the
    /// region to one of the linked lists
    /// of blocks.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Get the mutex lock on the allocator
        let mut allocator = self.lock();

        // Find out if there is a
        // big enough block size
        // to add to a linked list
        match list_index(&layout) {
            // If there is a size big
            // enough, create a new
            // node with the next node
            // set as the node in the
            // list heads
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };

                // Ensure that the block size
                // is big enough for the aligned
                // ListNode struct to insert
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                // Write the ListNode struct into
                // the newly freed block of memory,
                // and set the list_heads index
                // for the block size equal to the
                // newly freed block
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }

            // If there is no block size big
            // enough, add the free memory to
            // the fallback linked list allocator
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}
