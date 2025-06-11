//! Modern Syscall Interface for Polished OS
//!
//! This crate provides the syscall ABI and syscall handler for the kernel. It is designed for x86_64 and uses the `syscall` instruction (not legacy `int 0x80`).
//!
//! # Example
//! Add a syscall to return a constant value from userspace.

#![no_std]

/// Syscall numbers for the Polished OS ABI.
#[repr(u64)]
pub enum Syscall {
    /// Example: Return a constant value (SYSCALL_TEST)
    Test = 0,
    // Add more syscalls here
}

/// The syscall handler. This should be called from the kernel's syscall entry point.
///
/// # Safety
/// Must only be called with valid arguments from a syscall context.
pub unsafe fn syscall_handler(
    num: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
) -> u64 {
    match num {
        x if x == Syscall::Test as u64 => syscall_test(),
        // Add more syscall dispatches here
        _ => u64::MAX, // Unknown syscall
    }
}

/// Example syscall: returns 42.
pub fn syscall_test() -> u64 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_test() {
        assert_eq!(syscall_test(), 42);
    }
    #[test]
    fn test_syscall_handler_dispatch() {
        unsafe {
            assert_eq!(syscall_handler(Syscall::Test as u64, 0, 0, 0, 0, 0, 0), 42);
            assert_eq!(syscall_handler(999, 0, 0, 0, 0, 0, 0), u64::MAX);
        }
    }
}
