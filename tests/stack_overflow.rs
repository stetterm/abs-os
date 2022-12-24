//! Test module that ensures that
//! a stack overflow will cause
//! a double fault instead of
//! triple faulting and boot looping.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
  unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  abs_os::test_panic_handler(info);
}
