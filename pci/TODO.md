## Immediate Enhancements (High Priority)

### Bus-Wide Support

- [ ] Support scanning all buses (0–255), not just bus 0.
  - Walk all devices/functions per bus.
  - Detect and recurse into bridges (secondary buses).

### Multi-Function Device Support

- [ ] Check header type bit 7 to detect and scan functions 1–7 on multi-function devices.

### Struct-Based Device Model

- [ ] Define a `PciDevice` struct with fields: `bus`, `device`, `function`, `vendor_id`, `device_id`, `class`, `subclass`, `prog_if`, `header_type`.
- [ ] Provide `impl core::fmt::Debug` and a printable summary.

______________________________________________________________________

## Hardware Feature Access

### Configuration Space Read/Write

- [ ] Add `read_config_u8`, `read_config_u16`, `read_config_u32` (and `write_*` variants).
- [ ] Use those to clean up decoding logic.

### BAR (Base Address Registers) Parsing

- [ ] Add support for reading and interpreting device BARs (I/O vs MMIO, 32 vs 64-bit).
- [ ] Return size and type via probe by writing 0xFFFFFFFF and reading back.

______________________________________________________________________

## Logging and Metadata Improvements

### Better Vendor/Class Decoding

- [ ] Use `phf` or `match` tables to print known class and vendor names.
- [ ] Allow optional integration with `polished_hwdb` crate (shared vendor/class database).

### Configurable Logging

- [ ] Abstract logger: define a `Logger` trait and use generics or feature flags to support:
  - `serial_logging`
  - `log` crate (in hosted/tested environments)
  - No-op for silence

______________________________________________________________________

## Safety and API

### Safe Public API

- [ ] Expose a safe API for scanning and getting a list of `PciDevice` entries.
- [ ] Keep unsafe internals well-documented and minimal.

### Error Handling

- [ ] Introduce `Result` types for device reads and enumeration.
- [ ] Define errors: e.g., `DeviceNotFound`, `InvalidOffset`, `IoFailure`.

______________________________________________________________________

## Usability as a Crate

### Feature Flags and Optional Deps

- [ ] Add a `logger` feature that enables `serial_logging`, off by default.
- [ ] Add a `decode_names` feature to enable lookup tables for vendor/class.

### Docs & Examples

- [ ] Improve `lib.rs` with module-level docs showing the full usage flow.
- [ ] Provide integration examples for:
  - Kernel-level init
  - Dumping all PCI devices
  - Finding first network device

______________________________________________________________________

## Advanced/Long-Term Features

### Driver Binding Support

- [ ] Create traits or metadata for matching devices by class/vendor and binding to driver modules.

### ECAM Support (Modern PCIe)

- [ ] Abstract PCI config access to support both legacy I/O (0xCF8/CFC) and memory-mapped ECAM.

### Thread-Safety Option (optional)

- [ ] For hosted/tested use: add optional `spin::Mutex` or `critical_section` for lockable scan routines.

______________________________________________________________________

## Testing and CI

### Simulated PCI Config Space

- [ ] Add a "mock backend" feature for testing PCI enumeration in userland.
- [ ] Use `cfg(test)` or build-time flag to substitute access routines.

### CI Setup

- [ ] Build for `no_std`, `no_main`, and run cargo check for tests with/without logger.
