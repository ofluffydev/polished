# Minimal Kernel Paging System Setup (x86_64, Rust)

## Bootloader Paging Setup (Pre-Kernel)

### Basic Page Table Setup (4-Level Paging)

- [ ] Allocate a PML4 table (physically aligned 4 KiB page)
- [ ] Allocate a PDPT (PML3) table
- [ ] Set up an identity map (0x0000_0000_0000_0000):
  - [ ] Set PML4[0] → PDPT
  - [ ] Set PDPT[0] → 1GiB huge page (1:1 mapping of physical memory)
    - [ ] Use 0x0000_0000_0000_0000 | present | writable | huge
- [ ] Set up a higher-half mapping (e.g., `0xFFFF_9000_0000_0000`):
  - [ ] Use PML4[256] = PDPT
  - [ ] PDPT[0] = 1GiB huge page of physical memory (same frame)
- [ ] Setup recursive mapping:
  - [ ] Set PML4[511] = physical address of PML4 | present | writable
  - [ ] This enables you to reference the page tables themselves using special virtual addresses (e.g., `0xFFFF_FFFF_FFFF_F000`).
- [ ] Load CR3 with physical address of PML4
- [ ] Enable paging:
  - [ ] Set CR0.PG = 1, CR4.PAE = 1, EFER.LME = 1, etc.
  - [ ] Jump to higher-half kernel entry point (e.g., `0xFFFF_9000_0000_0000`)

## Kernel Paging Initialization

### Create a `OffsetPageTable` or manual mapper

- [ ] Use the recursive slot to get a virtual address to the PML4:
  - [ ] `let level_4_table: &mut PageTable = &mut *(0xFFFF_FFFF_FFFF_F000 as *mut PageTable);`
- [ ] Use that address to initialize an `OffsetPageTable`, if desired:
  - [ ]
    ```rust
    let mapper = unsafe { OffsetPageTable::new(level_4_table, phys_to_virt_offset) };
    ```

## Kernel Page Mapping Flow (Manual or via Mapper)

### Mapping a new page

- [ ] Select a virtual address (e.g., `0xFFFF_9000_0000_0000`)
- [ ] Allocate a physical frame for backing memory
- [ ] Walk or create the PML4 → PDPT → PD → PT chain using the recursive mapping:
  - [ ] PML4: `0xFFFF_FFFF_FFFF_F000 + 8 * index`
  - [ ] PDPT: `0xFFFF_FFFF_FF800000 + (index << 21)`
  - [ ] PD: `0xFFFF_FF8000000000 + (index << 30)`
  - [ ] PT: `0xFFFF_000000000000 + (index << 39)`
    - [ ] (This depends on the layout you assume; most crates hide this)
- [ ] Write entry in PT with:
  - [ ] `physical_address | PTE_FLAGS (present | writable | etc.)`
- [ ] Flush TLB entry for that virtual address:
  - [ ]
    ```rust
    x86_64::instructions::tlb::flush(VirtAddr::new(virt));
    ```
- [ ] Access the virtual address safely:
  - [ ]
    ```rust
    unsafe { core::ptr::write_volatile(ptr, value); }
    ```

## Debugging Checklist

- [ ] Did you set PML4[511] = PML4 physical address (recursive)?
- [ ] Did you use `invlpg`/`flush()` after writing a new PTE?
- [ ] Did your physical frame allocator hand you a clean, aligned page?
- [ ] Did you map all intermediate tables (PDPT, PD, PT) before final entry?
- [ ] Are you sure CR3 was loaded with the same PML4 as you’re using?
- [ ] Did you use correct flags: PRESENT | WRITABLE (and NO_EXEC where needed)?
- [ ] Does your memory map avoid reusing firmware/reserved memory?
- [ ] Are you using 4 KiB pages (not 2M/1G) when setting lower-level mappings?

## Quick Notes on Recursive Mapping

- [ ] With `PML4[511] = self`, you can reference all page tables via:
  - [ ] PML4 = `0xFFFF_FFFF_FFFF_F000`
  - [ ] PDPT = `0xFFFF_FFFF_FF800000`
  - [ ] PD = `0xFFFF_FF8000000000`
  - [ ] PT = `0xFFFF_000000000000`
  - [ ] These allow walking/modifying page tables without knowing physical addresses.
