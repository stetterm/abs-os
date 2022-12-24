//! Module for handling exceptions
//! that occur when executing code.
//! Without this module, the OS only
//! knows how to panic.

use crate::{gdt, println};

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// A single static interrupt
// descriptor table is creating
// for the operating system to
// call handler functions after
// faults/exceptions.
lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
      idt.double_fault.set_handler_fn(double_fault_handler)
          .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
  };
}

/// Initializes the interrupt descriptor
/// table with all the hanlder functions
/// called when an exception occurs.
pub fn init_idt() {
  IDT.load();
}

// Called when a breakpoint exception
// happens.
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
  println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Ensure that breakpoint exceptions
// cause the expected behavior.
#[test_case]
fn test_breakpoint_exception() {

  // Cause a breakpoint exception
  x86_64::instructions::interrupts::int3();
}

// Handler function called when
// a fault occurs during a fault
// handler function.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
  panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
