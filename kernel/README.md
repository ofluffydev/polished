# Polished Kernel TODO and Feature List

This document tracks the status and planned features for the Polished OS kernel. For project context, see the [main README](../README.md).

______________________________________________________________________

## Features (in order of implementation)

- [x] Bootstrapping and initialization
- [x] No-std and no-main environment
- [x] Naked entry point (assembly bootstrap)
- [x] Custom panic handler with serial output
- [x] Serial logging (info, warn, error)
- [x] Heap memory management (buddy_system_allocator)
- [x] Custom memory functions (memset, memcpy, memmove, memcmp)
- [x] Framebuffer support (logging, clearing, demo)
- [ ] Interrupt handling
- [ ] Timer management
- [ ] CPU context switching
- [ ] Memory management
- [ ] Virtual memory
- [ ] User and kernel mode separation
- [ ] System call interface
- [ ] Process scheduling
- [ ] Concurrency primitives
- [ ] Fault handling and recovery
- [ ] Device drivers
- [ ] Hardware abstraction
- [ ] I/O buffering and caching
- [ ] File system abstraction
- [ ] Inter-process communication
- [ ] Resource tracking and cleanup
- [ ] Security and permissions
- [ ] User authentication support
- [ ] Networking stack
- [ ] Power management
- [ ] Module loading/unloading
- [ ] System resource accounting
- [ ] Timekeeping and clock management
- [ ] Debugging facilities
- [ ] Logging and tracing

______________________________________________________________________

This list is updated as features are implemented. Contributions and suggestions are welcome!
