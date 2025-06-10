// This module sets up and loads the Global Descriptor Table (GDT) for x86_64 systems.
// The GDT defines the memory segments used by the CPU, including code and data segments for both kernel and user modes.

use once_cell::unsync::OnceCell;
use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};

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
