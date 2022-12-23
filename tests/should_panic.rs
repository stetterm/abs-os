//! Simple test module that
//! ensures the OS panics 
//! when it is supposed to
//! panic.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use abs_os::{QemuExitCode, exit_qemu, serial_println};

use core::panic::PanicInfo;

// Function called by a test
// that panics, which prints
// out a success message to
// stdout.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  serial_println!("[ok]");
  exit_qemu(QemuExitCode::Success);
  loop {}
}

// Starting point for the
// tests in this module
// that calls the test_main.
#[no_mangle]
pub extern "C" fn _start() -> ! {
  should_fail();
  serial_println!("[test did no panic]");
  exit_qemu(QemuExitCode::Failure);
  loop {}
}

use abs_os::serial_print;

fn should_fail() {
  serial_print!("should_panic::should_fail...\t");
  assert_eq!(0, 1);
}











