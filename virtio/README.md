# polished_virtio: A Beginner-Friendly Virtio Device Library for Rust OS Development

**polished_virtio** is a Rust library that makes it easy to interact with [Virtio](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html) devices in a safe, ergonomic, and no_std environment—perfect for hobby operating systems, kernels, or bare-metal projects. This crate is part of the [Polished OS](../README.md) project, but is designed to be reusable in any OS or low-level Rust project.

---

## What is Virtio?

Virtio is a standard for virtual devices, commonly used in virtual machines (like QEMU or KVM) to provide fast, efficient, and simple access to hardware-like devices such as network cards, block devices (disks), and more. Instead of emulating real hardware, Virtio provides a paravirtualized interface: the guest OS and the hypervisor cooperate for better performance and simplicity.

**Why does Virtio matter?**
- **Performance:** Virtio is much faster than traditional device emulation.
- **Simplicity:** The interface is designed to be easy to implement in both the guest and the hypervisor.
- **Portability:** The same Virtio interface works across different hypervisors and platforms.

If you're writing an OS or kernel that runs in QEMU, KVM, or other VMs, supporting Virtio is the easiest way to get working disk, network, and other I/O devices.

---

## What does this library do?

`polished_virtio` provides a safe, well-documented abstraction for accessing Virtio devices using port-mapped I/O (PIO). It helps you:
- **Discover and represent Virtio devices** found on the PCI bus
- **Read and write device registers** safely using Rust
- **Manage device status and features** (the handshake between driver and device)
- **Select and configure device queues** (for data transfer)
- **Reset and initialize Virtio devices** in the correct order

All low-level hardware access is marked `unsafe` and carefully documented, so you know exactly what the risks are and when it's safe to use each method.

---

## Who is this for?

- **Beginner OS developers** who want to add Virtio support to their kernel
- **Rustaceans** building no_std or bare-metal projects
- **Anyone** who wants a clear, well-documented example of Virtio device access in Rust

You do **not** need to be an expert in PCI, hardware, or virtualization to use this crate! The documentation explains each step and why it matters.

---

## How does it work?

### 1. Representing a Virtio Device

A Virtio device is represented by the `VirtioDevice` struct, which stores its PCI location and I/O base address. This is all you need to access its registers.

```rust
let virtio = VirtioDevice::new(bus, device, function, io_base);
```

### 2. Reading and Writing Registers

Virtio devices are controlled by reading and writing to specific registers (offsets from the I/O base). This library provides safe wrappers for all standard registers, using enums for clarity.

```rust
// Read the device's feature bits
let features = unsafe { virtio.features() };

// Write back the features you support
unsafe { virtio.write_features(features) };
```

### 3. Device Initialization Sequence

Virtio devices require a specific handshake to initialize:
1. **Reset** the device
2. **Acknowledge** the device
3. Set the **DRIVER** status
4. **Negotiate features**
5. Set **FEATURES_OK**
6. Set **DRIVER_OK**

This crate provides methods for each step, and a one-call `init()` method that does the full sequence for you.

```rust
if virtio.init().is_ok() {
    // Device is ready!
}
```

### 4. Safety and `unsafe`

Direct hardware access is always `unsafe` in Rust. This library marks all such methods as `unsafe` and documents exactly when and how to use them. If you're new to OS dev, read the docs for each method before calling it.

---

## Example: Detecting and Initializing a Virtio Device

```rust
// Suppose you have a PCI device (from polished_pci)
let pci_dev = ...;
let virtio = VirtioDevice::from(pci_dev);

// Reset and initialize the device
unsafe { virtio.reset(); }
unsafe { virtio.acknowledge(); }
unsafe { virtio.driver(); }
let features = unsafe { virtio.features() };
unsafe { virtio.write_features(features); }
unsafe { virtio.features_ok(); }
unsafe { virtio.driver_ok(); }
```

Or, use the convenience method:

```rust
if virtio.init().is_ok() {
    // Ready to use!
}
```

---

## Why is this needed?

- **Most OS tutorials skip Virtio** or provide only C code. This crate gives you a modern, idiomatic Rust interface.
- **PCI and Virtio are tricky** for beginners. This crate abstracts the details and provides clear, safe APIs.
- **No_std and bare-metal friendly:** No heap, no stdlib, just pure Rust and hardware access.

---

## Features
- PCI device abstraction (with [polished_pci](../pci/))
- Safe, documented register access
- Full Virtio initialization sequence
- Support for legacy (PIO) Virtio devices
- Designed for extensibility (block, net, etc. drivers can build on this)

---

## Limitations
- Only supports **legacy (port-mapped I/O) Virtio devices** for now (the kind QEMU provides by default)
- Does **not** implement virtqueues or actual block/network drivers yet—this is just the device abstraction layer (yet)
- Assumes you are running in a privileged, bare-metal, or kernel environment (not userland)

---

## License

This crate is licensed under the [zlib License](https://zlib.net/zlib_license.html).

---

## Further Reading
- [Virtio Spec (OASIS)](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)
- [Writing a Virtio Driver (osdev.org)](https://wiki.osdev.org/Virtio)
- [Polished OS Project](../README.md)

---

## Contributing

Contributions, bug reports, and questions are welcome! If you're new to OS dev or Rust, feel free to open an issue or PR—this project is beginner-friendly.
