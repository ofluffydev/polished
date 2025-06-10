#![no_std]

// This module sets up and loads the Global Descriptor Table (GDT) for x86_64 systems.
// The GDT defines the memory segments used by the CPU, including code and data segments for both kernel and user modes.

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
#[repr(align(16))]
struct AlignedStacks([[u8; IST_STACK_SIZE]; IST_ENTRIES]);

static mut IST_STACKS: AlignedStacks = AlignedStacks([[0; IST_STACK_SIZE]; IST_ENTRIES]);

/// Static OnceCell for the TSS
static mut TSS: OnceCell<TaskStateSegment> = OnceCell::new();

/// Returns a reference to the TSS, initializing it if needed.
pub fn get_tss() -> &'static TaskStateSegment {
    unsafe {
        #[allow(static_mut_refs)] // Allowed because OnceCell is used
        TSS.get_or_init(|| {
            let mut tss = TaskStateSegment::new();
            // Set IST1 for double fault
            tss.interrupt_stack_table[1] = {
                let stack_start = &IST_STACKS.0[1] as *const u8 as u64;
                let stack_end = stack_start + IST_STACK_SIZE as u64;
                VirtAddr::new(stack_end)
            };
            // Set IST2 for NMI
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
