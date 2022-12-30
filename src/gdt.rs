//! Module for the Task State
//! Segment (TSS). This is used
//! for storing information about
//! the previous stack and program
//! context.

use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

// Index into Interrupt Stack Table
// that stores the Task State
// Segment information.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// One static Task State Segment is used
// across the operating system. It stores
// stack information about tasks when
// stack switching occurs.
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};

// A static Global Descriptor Table
// is used across the operating
// system. This is used for storing
// memory segments (no longer supported)
// and for switching between kernal and
// user space, as well as loading a TSS
// that stores a stack.
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

use x86_64::structures::gdt::SegmentSelector;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// Initialize the Global Descriptor
// Table that holds a reference to the
// static TSS and code selector.
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
