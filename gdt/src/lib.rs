//! # gdt
//!
//! This module sets up and loads the Global Descriptor Table (GDT) for x86_64 systems.
//!
//! ## What is the GDT?
//!
//! The Global Descriptor Table (GDT) is a fundamental data structure used by x86 CPUs to define the characteristics of the various memory segments used by the system. Each entry in the GDT describes a segment, including its base address, size, access permissions, and type (code, data, etc). In modern 64-bit (x86_64) systems, segmentation is mostly unused, but the GDT is still required for certain features:
//!
//! - Defining code and data segments for kernel and user modes
//! - Setting up the Task State Segment (TSS) for handling interrupts and exceptions
//! - Providing separate stacks for critical exceptions (via the Interrupt Stack Table, IST)
//!
//! ## Why do we need a GDT?
//!
//! Even though 64-bit mode uses a flat address space, the CPU still requires a GDT to be loaded. The GDT is also necessary for features like privilege separation (kernel vs user), and for configuring the TSS, which is used for stack switching on interrupts.
//!
//! ## How does this module work?
//!
//! - Statically allocates and initializes the GDT and TSS
//! - Sets up segment descriptors for kernel and user code/data
//! - Configures the TSS with dedicated stacks for critical exceptions (double fault, NMI)
//! - Loads the GDT and updates the segment registers
//!
//! This is typically called early in kernel initialization, before enabling interrupts.

#![no_std]

use once_cell::unsync::OnceCell;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

/// Static OnceCell for the GDT and its segment selectors.
///
/// The tuple contains:
/// - The `GlobalDescriptorTable` instance
/// - An array of four `SegmentSelector`s for:
///   0. Kernel code segment
///   1. Kernel data segment
///   2. User code segment
///   3. User data segment
static mut GDT: OnceCell<(GlobalDescriptorTable, [SegmentSelector; 4])> = OnceCell::new();

/// Number of IST entries (x86_64 supports up to 7)
const IST_ENTRIES: usize = 3; // 0: unused, 1: double fault, 2: NMI (add more as needed)

/// Size of each IST stack (8 KiB is typical)
const IST_STACK_SIZE: usize = 4096 * 2;

/// Statically allocate stacks for IST
///
/// The Interrupt Stack Table (IST) allows the CPU to switch to a dedicated stack when handling certain critical exceptions (like double faults or NMIs).
#[repr(align(16))]
struct AlignedStacks([[u8; IST_STACK_SIZE]; IST_ENTRIES]);

static mut IST_STACKS: AlignedStacks = AlignedStacks([[0; IST_STACK_SIZE]; IST_ENTRIES]);

/// Static OnceCell for the TSS (Task State Segment)
///
/// The TSS is a special structure used by the CPU to store information about a task, including pointers to stacks for handling interrupts.
static mut TSS: OnceCell<TaskStateSegment> = OnceCell::new();

/// Returns a reference to the TSS, initializing it if needed.
///
/// The TSS is set up with dedicated stacks for double fault and NMI exceptions using the IST.
pub fn get_tss() -> &'static TaskStateSegment {
    unsafe {
        #[allow(static_mut_refs)] // Allowed because OnceCell is used
        TSS.get_or_init(|| {
            let mut tss = TaskStateSegment::new();
            // Set IST1 for double fault (critical error stack)
            tss.interrupt_stack_table[1] = {
                let stack_start = &IST_STACKS.0[1] as *const u8 as u64;
                let stack_end = stack_start + IST_STACK_SIZE as u64;
                VirtAddr::new(stack_end)
            };
            // Set IST2 for NMI (non-maskable interrupt stack)
            tss.interrupt_stack_table[2] = {
                let stack_start = &IST_STACKS.0[2] as *const u8 as u64;
                let stack_end = stack_start + IST_STACK_SIZE as u64;
                VirtAddr::new(stack_end)
            };
            tss
        })
    }
}

/// Initializes and loads the Global Descriptor Table (GDT).
///
/// # Safety
/// This function is safe to call only once and before interrupts are enabled, as it modifies segment registers.
///
/// The GDT is initialized with four segments:
/// - Kernel code
/// - Kernel data
/// - User code
/// - User data
///
/// After loading the GDT, the segment registers (CS, SS, DS, ES) are set to the appropriate selectors.
///
/// # How it works
/// 1. Initializes the GDT and appends descriptors for kernel/user code and data segments.
/// 2. Appends a TSS descriptor, which is required for stack switching on interrupts.
/// 3. Loads the GDT into the CPU using the `lgdt` instruction.
/// 4. Updates the segment registers to use the new selectors.
///
/// # Example
/// ```ignore
/// gdt::init_gdt();
/// ```
pub fn init_gdt() {
    // Safety: single-threaded, called only once before interrupts enabled.
    let (gdt, selectors) = unsafe {
        #[allow(static_mut_refs)]
        GDT.get_or_init(|| {
            let mut gdt = GlobalDescriptorTable::new();
            // Append kernel code segment (index 1, selector 0x08)
            let code_sel = gdt.append(Descriptor::kernel_code_segment());
            // Append kernel data segment (index 2, selector 0x10)
            let data_sel = gdt.append(Descriptor::kernel_data_segment());
            // Append user code segment (index 3, selector 0x18)
            let user_code_sel = gdt.append(Descriptor::user_code_segment());
            // Append user data segment (index 4, selector 0x20)
            let user_data_sel = gdt.append(Descriptor::user_data_segment());
            // Append TSS descriptor (index 5, selector 0x28)
            let tss = get_tss();
            gdt.append(Descriptor::tss_segment(tss));
            (gdt, [code_sel, data_sel, user_code_sel, user_data_sel])
        })
    };
    gdt.load();

    unsafe {
        // Set all segment registers that might be used during interrupts
        CS::set_reg(selectors[0]); // kernel code segment
        SS::set_reg(selectors[1]); // kernel data segment
        DS::set_reg(selectors[1]);
        ES::set_reg(selectors[1]);
    }
}
