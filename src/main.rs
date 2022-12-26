//! Main entry point of 
//! abs_os kernel.


#![feature(custom_test_frameworks)] // Enable support for
#![test_runner(abs_os::test_runner)] // custom test runner
#![reexport_test_harness_main = "test_main"] // Use test_main as the
                                             // name of function called
                                             // to run all tests
#![no_main] // Disable rust entry point/runtime
#![no_std]  // Disable rust standard lib

use core::panic::PanicInfo;
use abs_os::println;

// Main entry point function
#[no_mangle]    // Function is called _start
pub extern "C" fn _start() -> ! {

  println!("Hello World{}", "!");

  // Initialize the interrupt descriptor
  // table necessary for handling exceptions.
  abs_os::init();

  use x86_64::registers::control::Cr3;

  let (level_4_page_table, _) = Cr3::read();
  println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
   
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


