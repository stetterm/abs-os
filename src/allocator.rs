//! Implementation of global
//! heap memory allocator.

use fixed_size_block::FixedSizeBlockAllocator;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

// Static global memory allocator
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = 
    Locked::new(FixedSizeBlockAllocator::new());

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

/// Wrapper around mutex so traits can be
/// implemented on the A type wrapped in
/// a mutex.
pub struct Locked<A> {
  inner: spin::Mutex<A>,
}

impl<A> Locked<A> {

  /// Wrap inner A type in mutex
  pub const fn new(inner: A) -> Self {
    Locked {
      inner: spin::Mutex::new(inner),
    }
  }

  /// Acquire the mutex lock on the inner type
  pub fn lock(&self) -> spin::MutexGuard<A> {
    self.inner.lock()
  }
}

// ADDRESS ALIGNMENT FOR ALLOCATOR

/// Aligns the memory address to the next
/// highest byte-aligned address.
fn align_up(addr: usize, align: usize) -> usize {
  (addr + align - 1) & !(align - 1)
}





























