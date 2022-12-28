//! Implementation of linked list
//! heap allocator using cons list
//! of heap allocations.

use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use super::{align_up, Locked};

/// Node of the linked list that
/// stores the size of an allocation
/// and the next allocation in the list.
struct ListNode {
  size: usize,
  next: Option<&'static mut ListNode>,
}

impl ListNode {

  /// Creates a new node with no
  /// following node.
  const fn new(size: usize) -> Self {
    ListNode { size, next: None }
  }

  /// Get the start address of
  /// the heap allocation.
  fn start_addr(&self) -> usize {
    self as *const Self as usize
  }

  /// Get the end of the 
  /// heap allocation.
  fn end_addr(&self) -> usize {
    self.start_addr() + self.size
  }
}

/// Allocator that implements the
/// linked list strategy by storing
/// the first link of the heap
pub struct LinkedListAllocator {
  head: ListNode,
}

impl LinkedListAllocator {
 
  /// Produces a new linked list
  /// heap with an empty
  /// head link
  pub const fn new() -> Self {
    Self {
      head: ListNode::new(0),
    }
  }

  /// Initializes the linked list allocator
  /// with the start and end addresses of the heap
  pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    self.add_free_region(heap_start, heap_size);
  }

  /// Add the memory region provided to the
  /// start of the linked list
  unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
    
    // Ensure there is enough memory 
    // for the ListNode
    assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
    assert!(size >= mem::size_of::<ListNode>());

    // Create a new ListNode and set the
    // next value to the head link
    let mut node = ListNode::new(size);
    node.next = self.head.next.take();

    // Write the new node into
    // the newly allocated memory
    let node_ptr = addr as *mut ListNode;
    node_ptr.write(node);
    self.head.next = Some(&mut *node_ptr)
  }

  fn find_region(&mut self, size: usize, align: usize)
      -> Option<(&'static mut ListNode, usize)>
  {

    // Start at the head of the linked list
    let mut current = &mut self.head;

    // Iterate until a valid region of
    // memory is found for a link
    while let Some(ref mut region) = current.next {

      // If the region is valid and large
      // enough, the region is returned
      if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
        let next = region.next.take();
        let ret = Some((current.next.take().unwrap(), alloc_start));
        current.next = next;
        return ret;

      // Otherwise, move to the next link
      } else {
        current = current.next.as_mut().unwrap();
      }
    }

    // No valid region could be found
    None
  }

  fn alloc_from_region(region: &ListNode, size: usize, align: usize)
      -> Result<usize, ()>
  {

    // Get the start and end address of
    // the allocated region
    let alloc_start = align_up(region.start_addr(), align);
    let alloc_end = alloc_start.checked_add(size).ok_or(())?;

    // The region is not large enough
    if alloc_end > region.end_addr() {
      return Err(());
    }

    // Get the amount of memory that is
    // left over after this allocation is made.
    // If there is not enough memory for a 
    // ListNode, error is returned.
    let excess_size = region.end_addr() - alloc_end;
    if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
      return Err(());
    }

    // Region can be allocated
    Ok(alloc_start)
  }

  /// Returns an adjusted size and alignment
  /// for a layout so that it can store a ListNode.
  fn size_align(layout: Layout) -> (usize, usize) {
    
    // Get the adjusted, aligned version
    // of the provided layout. If it is
    // not big enough, it is padded by the
    // pad_to_align function.
    let layout = layout
        .align_to(mem::align_of::<ListNode>())
        .expect("adjusting alignment failed")
        .pad_to_align();

    // Adjusts the size to be atleast big
    // enough to store a ListNode.
    let size = layout.size().max(mem::size_of::<ListNode>());

    // Return the new alignment
    (size, layout.align())
  }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {

  /// Allocates the provided region in the heap
  /// using a linked list of ListNode structs.
  /// A null pointer is returned on failure.
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    
    // Get the size and alignment of
    // the provided layout, and get
    // the lock on the global allocator.
    let (size, align) = LinkedListAllocator::size_align(layout);
    let mut allocator = self.lock();

    if let Some((region, alloc_start)) = allocator.find_region(size, align) {
      let alloc_end = alloc_start.checked_add(size).expect("overflow");
      let excess_size = region.end_addr() - alloc_end;
      if excess_size > 0 {
        allocator.add_free_region(alloc_end, excess_size);
      }
      alloc_start as *mut u8
    } else {
      ptr::null_mut()
    }
  }

  /// Deallocate the provided layout 
  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let (size, _) = LinkedListAllocator::size_align(layout);

    self.lock().add_free_region(ptr as usize, size)
  }
}





















