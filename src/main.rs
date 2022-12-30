//! Main entry point of
//! abs_os kernel.

#![feature(custom_test_frameworks)] // Enable support for
#![test_runner(abs_os::test_runner)] // custom test runner
#![reexport_test_harness_main = "test_main"] // Use test_main as the
// name of function called
// to run all tests
#![no_main] // Disable rust entry point/runtime
#![no_std] // Disable rust standard lib

extern crate alloc;

use abs_os::{
    println,
    task::{keyboard, executor::Executor, Task},
};

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

// Main entry point function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use abs_os::{
        allocator,
        memory::{self, BootInfoFrameAllocator},
    };
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");

    // Initialize the interrupt descriptor
    // table necessary for handling exceptions.
    abs_os::init();

    // Get the level 4 page table
    // using the memory module in
    // src/memory.rs
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("failed to initialize heap");

    #[cfg(test)]
    test_main();
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    println!("abs_os did not crash");
    abs_os::hlt_loop();
}

async fn async_number() -> u32 {
  42
}

async fn example_task() {
  let number = async_number().await;
  println!("async number: {}", number);
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
