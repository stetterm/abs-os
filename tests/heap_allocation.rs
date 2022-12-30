//! Integration tests to ensure that
//! heap allocations are made properly.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(abs_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use abs_os::allocator::HEAP_SIZE;
use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(main);

/// Called when the heap allocation
/// tests are run in this module.
/// It sets up the OS to test
/// memory allocations.
fn main(boot_info: &'static BootInfo) -> ! {
    use abs_os::{
        allocator,
        memory::{self, BootInfoFrameAllocator},
    };
    use x86_64::VirtAddr;

    // Initialize the OS and the
    // frame allocator with the
    // physical memory offset
    // provided by the bootloader
    abs_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Initialize the heap using the
    // frame allocator and the memory
    // mapper created.
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // Run the tests
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    abs_os::test_panic_handler(info)
}

//// TESTS

// Two simple allocations are
// made and the expected values
// are asserted on.
#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(41, *heap_value_1);
    assert_eq!(13, *heap_value_2);
}

// A vector is created and the
// sum of the elements is compared
// to the expected value.
#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

// Many box allocations are made
// and the stored values are
// compared against the expected values.
// This ensures that memory is
// properly reused after being freed
// when the variable x goes out of scope.
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

// Tests that memory is able to
// be preserved through many
// allocations and frees in a loop.
#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}
