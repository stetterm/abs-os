
// src/main.rs 
// main entry point of 
// the rust kernel


#![feature(custom_test_frameworks)] // Enable support for
#![test_runner(crate::test_runner)] // custom test runner
#![reexport_test_harness_main = "test_main"]
#![no_main] // Disable rust entry point/runtime
#![no_std]  // Disable rust standard lib


use core::panic::PanicInfo;

// Called on panic
#[cfg(not(test))] // User different panic for tests
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failure);
  loop {}
}

// Main entry point function
#[no_mangle]    // Function is called _start
pub extern "C" fn _start() -> ! {

  println!("Hello World{}", "!");

  #[cfg(test)]
  test_main();

  loop {}
}

mod vga_buffer;
mod serial;

// Test runner runs each test
// provided in the slice and
// sends the success signal
// to Qemu
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
  serial_println!("Running {} tests", tests.len());
  for test in tests {
    test();
  }

  exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial() {
  serial_print!("trivial assertion... ");
  assert_eq!(1, 1);
  serial_println!("[ok]");
}

#[test_case]
fn failed_test() {
  serial_print!("incorrect assertion... ");
  assert_eq!(1, 2);
  serial_println!("[ok]");
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






















