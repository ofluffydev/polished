# Polished GDT Library

**Polished GDT** is a Rust library that provides initialization and management of the Global Descriptor Table (GDT) for x86_64 systems. It is a core component of the Polished OS project, responsible for setting up essential CPU segmentation and privilege separation required for safe and correct kernel operation.

______________________________________________________________________

## What is the GDT?

The **Global Descriptor Table (GDT)** is a fundamental data structure in the x86 and x86_64 architectures. It defines the characteristics of the various memory segments used by the CPU, such as their base addresses, sizes, access privileges, and types (code, data, stack, etc.).

While modern 64-bit operating systems use a flat memory model (where segmentation is mostly disabled), the GDT is still required for:

- **Defining code and data segments**: Even in 64-bit mode, the CPU expects segment selectors to reference valid GDT entries.
- **Setting up the Task State Segment (TSS)**: The TSS is used for stack switching during interrupts and privilege transitions (e.g., from user mode to kernel mode).
- **Privilege separation**: The GDT allows the OS to define different privilege levels (ring 0 for kernel, ring 3 for user), which is essential for security and stability.

______________________________________________________________________

## Why is the GDT Needed?

- **CPU Requirement**: The x86_64 CPU requires a valid GDT to be loaded before entering long mode (64-bit mode). Without it, the CPU cannot correctly interpret segment selectors, leading to faults or undefined behavior.
- **Interrupt Handling**: The GDT is used to set up the TSS, which is critical for handling interrupts and exceptions safely, especially when switching stacks.
- **Kernel/User Separation**: The GDT enables the OS to enforce privilege boundaries between kernel and user code, preventing user applications from accessing sensitive kernel memory.

______________________________________________________________________

## What Does This Library Do?

The `polished_gdt` library provides:

- **Definition of GDT entries**: Code, data, and TSS segments are defined according to x86_64 requirements.
- **Initialization routines**: Functions to set up the GDT and load it into the CPU using the `lgdt` instruction.
- **TSS setup**: Creation and registration of the Task State Segment for safe interrupt stack switching.
- **Safe Rust abstractions**: The library uses Rust's type system and safety guarantees to minimize the risk of errors in this low-level code.

______________________________________________________________________

## How the GDT Works (in x86_64)

1. **Segment Descriptors**: Each entry in the GDT describes a segment (code, data, TSS, etc.) with its base, limit, and access flags.
1. **Segment Selectors**: When the CPU accesses memory, it uses segment selectors (indexes into the GDT) to determine which segment's rules apply.
1. **Long Mode**: In 64-bit mode, segmentation is mostly ignored (base is forced to 0, limit is ignored), but the GDT must still be present and valid.
1. **TSS and IST**: The TSS entry in the GDT points to a structure that holds stack pointers for different privilege levels and interrupt stack tables (ISTs).
1. **Loading the GDT**: The OS loads the GDT using the `lgdt` instruction, and updates segment registers to reference the new descriptors.

______________________________________________________________________

## Usage

Typically, you will call the GDT initialization function early in your kernel's startup sequence, before enabling interrupts or switching to user mode. Example usage:

```rust
// In your kernel initialization code:
polished_gdt::init();
```

This will:

- Define the required GDT entries (code, data, TSS)
- Load the GDT into the CPU
- Set up the TSS for safe interrupt handling

______________________________________________________________________

## Safety and Best Practices

- **Must be called before enabling interrupts**: The GDT and TSS must be set up before the CPU can safely handle interrupts or exceptions.
- **Do not modify the GDT after loading**: Once loaded, the GDT should remain unchanged unless you are performing a controlled update (e.g., for context switching).
- **Use provided abstractions**: The library provides safe wrappers for GDT and TSS setup; avoid writing raw assembly unless necessary.

______________________________________________________________________

## Further Reading

- [OSDev Wiki: Global Descriptor Table](https://wiki.osdev.org/Global_Descriptor_Table)
- [Intel® 64 and IA-32 Architectures Software Developer’s Manual, Vol. 3A: System Programming Guide](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [x86_64 crate documentation](https://docs.rs/x86_64/latest/x86_64/structures/gdt/index.html)

______________________________________________________________________

## License

This library is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the root of the repository for details.

______________________________________________________________________

**Polished GDT** is part of the [Polished OS](../README.md) project. Contributions and feedback are welcome!
