
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;

/// Trait that test runners
/// can use to run a test
/// and print out information
/// about the success or
/// failure of the test.
pub trait Testable {

  /// This function can be
  /// implemented to print
  /// testing information and
  /// run the actual test.
  fn run(&self) -> ();
}

// This implements the ability
// for any function type that
// takes no parameters and
// returns no values to
// be run with information
// about its success or failure.
impl<T> Testable for T
where
    T: Fn()  
{ 
    /// Implemented function to print
    /// information and run the T
    /// function type.
    fn run(&self) {
      serial_print!("{}...\t", core::any::type_name::<T>());
      self();
      serial_println!("[ok]");
    }
}

// Test runner runs each test
// provided in the slice and
// sends the success signal
// to Qemu, or runs the test
// panic function defined above.
pub fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} tests", tests.len());
  for test in tests {
    test.run();
  }

  exit_qemu(QemuExitCode::Success);
}

// Code is separated from the panic function
// below so that the common functionality
// can be used in other modules.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failure);
  loop {}
}

// Lib used for testing needs its
// own entry point to execute tests
// from test_main.
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
  test_main();
  loop {}
}

// Panic function called when a
// test panics and fails.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  test_panic_handler(info);
}

// Exit code signals to send to
// Qemu to close the VM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
  Success = 0x10,
  Failure = 0x11,
}

// Write the exit code to 0xf4, the
// address for Qemu exit codes
pub fn exit_qemu(exit_code: QemuExitCode) {
  use x86_64::instructions::port::Port;
  unsafe {
    let mut port = Port::new(0xf4);
    port.write(exit_code as u32);
  }
}








