//! Main entry point of 
//! abs_os kernel.


#![feature(custom_test_frameworks)] // Enable support for
#![test_runner(abs_os::test_runner)] // custom test runner
#![reexport_test_harness_main = "test_main"] // Use test_main as the
                                             // name of function called
                                             // to run all tests
#![no_main] // Disable rust entry point/runtime
#![no_std]  // Disable rust standard lib

use abs_os::println;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

// Main entry point function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
  use abs_os::memory::{self, BootInfoFrameAllocator};
  use x86_64::{
      structures::paging::{Page, Translate},
      VirtAddr,
  };

  println!("Hello World{}", "!");

  // Initialize the interrupt descriptor
  // table necessary for handling exceptions.
  abs_os::init();

  // Get the level 4 page table
  // using the memory module in
  // src/memory.rs
  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  let mut mapper = unsafe { memory::init(phys_mem_offset) };
  let mut frame_allocator = unsafe {
    BootInfoFrameAllocator::init(&boot_info.memory_map)
  };

  // Create a mapping using some large 
  // virtual address (0xdeadbeaf000)
  let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
  memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

  // Write at the VGA buffer offset
  // into the mapping that was created.
  // This will effectively print out
  // the value in write_volatile to the screen.
  let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
  unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

  #[cfg(test)]
  test_main();

  println!("abs_os did not crash");
  abs_os::hlt_loop();
}

// Called on panic
#[cfg(not(test))] // User different panic for tests
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  abs_os::hlt_loop();
}

// Panic function for main calls
// the test_panic_handler defined
// in src/lib.rs.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  abs_os::test_panic_handler(info);
}


