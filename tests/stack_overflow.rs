//! Test module that ensures that
//! a stack overflow will cause
//! a double fault instead of
//! triple faulting and boot looping.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use abs_os::serial_print;

use core::panic::PanicInfo;

// Function called when a panic
// occurs that runs the panic
// handler defined in src/lib.rs
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    abs_os::test_panic_handler(info);
}

// Entry point for the stack overflow
// test that initializes the OS and
// triggers a stack overflow. If the
// interrupt handling succeeds, the
// panic
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    // Initialize GDT for
    // the tests
    abs_os::gdt::init();
    init_test_idt();

    // Cause a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

// Stack overflow function that
// prevents itself from being
// optimized out at runtime.
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

// Instantiate a static IDT used
// for testing stack overflow and
// using a custom double fault
// function.
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(abs_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

// Test function called by the entry
// point to this test module (_start).
pub fn init_test_idt() {
    TEST_IDT.load();
}

use abs_os::{exit_qemu, serial_println, QemuExitCode};
use x86_64::structures::idt::InterruptStackFrame;

// Override of the x86 interrupt
// function called when a double
// fault occurs.
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
