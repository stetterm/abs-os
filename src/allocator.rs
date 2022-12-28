//! Implementation of global
//! heap memory allocator.

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

///// Dummy implementation of allocator
///// that returns a null pointer when
///// the allocator tries to allocate 
///// memory, and panics when an attempt
///// to deallocate occurs. This effectively
///// causes all allocations to fail.
//pub struct Dummy;
//
//unsafe impl GlobalAlloc for Dummy {
//  unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
//    null_mut()
//  }
//
//  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
//    panic!("dealloc should never be called");
//  }
//}
//
//#[global_allocator]
//static ALLOCATOR: Dummy = Dummy;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Constants used for setting
/// the range for heap allocations
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

/// Initializes the heap using the
/// provided mapper and allocator
/// to the range provided by the
/// above constants.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {

  // Get the range of the pages that
  // are in the range provided in the
  // above constants.
  let page_range = {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);
    Page::range_inclusive(heap_start_page, heap_end_page)
  };

  // For each page, allocate a
  // frame and map the corresponding
  // page to the frame.
  // If any of these allocations
  // fail, return MapToError from
  // the function.
  for page in page_range {
    let frame = frame_allocator
        .allocate_frame()
        .ok_or(MapToError::FrameAllocationFailed)?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe {
      mapper.map_to(page, frame, flags, frame_allocator)?.flush()
    };
  }

  // Initialize the heap allocator
  // using the heap size and start
  // constants
  unsafe {
    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
  }

  Ok(())
}






























