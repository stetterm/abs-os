
// src/main.rs 
// main entry point of 
// the rust kernel


#![no_main] // Disable rust entry point/runtime
#![no_std]  // Disable rust standard lib

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

// Main entry point function
#[no_mangle]    // Function is called _start
pub extern "C" fn _start() -> ! {

  //let vga_buffer = 0xb8000 as *mut u8;

  //for (i, &byte) in HELLO.iter().enumerate() {
  //    unsafe {
  //        *vga_buffer.offset(i as isize * 2) = byte;
  //        *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
  //    }
  //}

  println!("Hello World{}", "!");

  loop {}
}

mod vga_buffer;
