//! Integration tests that ensure
//! that the operating system
//! boots properly.

#![no_std]
#![no_main]
#![test_runner(abs_os::test_runner)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

// Entry point for the integration
// tests. These test the proper
// booting and features of the
// operating system kernel.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

// When a panic is called from
// these integration tests, it
// calls the test_panic_handler
// function defined in src/lib.rs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    abs_os::test_panic_handler(info);
}

use abs_os::println;

// Simple test to print out a string
// after the system has booted.
#[test_case]
fn test_println() {
    println!("test_println output");
}
