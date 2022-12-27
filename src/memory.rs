//! Module for controlling the physical
//! frames of memory and the page tables
//! used by processses and threads.

// INITIALIZE LEVEL 4 TABLE WITH PHYSADDR

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};


/// Initialize the page tables using
/// an offset between the virtual and
/// physical addresses. This is called
/// once when initializing the OS, and
/// it returns a page table with a 
/// static lifetime.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
  let level_4_table = active_level_4_table(physical_memory_offset);
  OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Get a reference to the active level
/// 4 page table. This is accomplished
/// by getting the virtual address mapping
/// of the physical address, which is 
/// captured by the kernel upon booting.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
  use x86_64::registers::control::Cr3;

  // Cr3 register holds the frame
  // of the level 4 page table
  let (level_4_table_frame, _) = Cr3::read();

  // Get the memory address of the
  // mapped page associated with
  // the level 4 table frame.
  let phys = level_4_table_frame.start_address();
  let virt = physical_memory_offset + phys.as_u64();
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

  &mut *page_table_ptr
}


/// Creates an example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

//// FRAME ALLOCATORS

// EMPTY FRAME ALLOCATOR

/// Empty FrameAllocator that always
/// returns none when a map_to call is made
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
  
  /// Allocates a new frame in the
  /// physical address space. If
  /// a new page table mus{t be created,
  /// the physical frame is returned
  /// wrapped in Some. Otherwise
  /// None is returned.
  ///
  /// self:     EmptyFrameAllocator
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    None
  }
}

// BOOTINFO FRAME ALLOCATOR

/// Stores the memory map from
/// the bootloader and the
/// index of the next usable frame index
pub struct BootInfoFrameAllocator {
  memory_map: &'static MemoryMap,
  next: usize,
}

impl BootInfoFrameAllocator {
  
  /// Initialize the memory map info
  /// passed to the kernel from the
  /// bootloader. The next usable
  /// frame index is started at 0.
  pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
    BootInfoFrameAllocator {
      memory_map,
      next: 0,
    }
  }

  /// Returns an iterator over the
  /// usable frames passed to the
  /// kernel from the bootloader.
  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
    self.memory_map.iter()

      // Get regions of memory that
      // are usable
      .filter(|r| r.region_type == MemoryRegionType::Usable)
      // Create a range iterator for
      // each of the available
      // regions of memory
      .map(|r| r.range.start_addr()..r.range.end_addr())

      // Flatten the 2D iterator of ranges
      // into a 1D iterator of 4KB pages
      .flat_map(|r| r.step_by(4096))

      // Return an iterator of PhysFrames
      .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
  }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
  
  /// Allocates a new frame using
  /// the BootInfoFrameAllocator.
  /// This uses the memory mapping
  /// passed to the kernel from
  /// the bootloader.
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    let frame = self.usable_frames().nth(self.next);
    self.next += 1;
    frame
  }
}

























