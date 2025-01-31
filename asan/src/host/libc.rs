//! # linux
//! The `LibcHost` supports the established means of interacting with the QEMU
//! emulator on Linux by means of issuing a bespoke syscall via the libc library
use {
    crate::{
        host::{Host, HostAction},
        shadow::PoisonType,
        GuestAddr,
    },
    core::ffi::c_long,
    libc::syscall,
    thiserror::Error,
};

#[derive(Debug)]
pub struct LibcHost;

impl Host for LibcHost {
    type Error = LibcHostError;

    fn load(start: GuestAddr, len: usize) -> Result<(), LibcHostError> {
        unsafe {
            let ret = syscall(Self::SYSCALL_NO, HostAction::CheckLoad as usize, start, len);
            if ret != 0 {
                return Err(LibcHostError::SyscallError(ret));
            }
        }
        Ok(())
    }

    fn store(start: GuestAddr, len: usize) -> Result<(), LibcHostError> {
        unsafe {
            let ret = syscall(
                Self::SYSCALL_NO,
                HostAction::CheckStore as usize,
                start,
                len,
            );
            if ret != 0 {
                return Err(LibcHostError::SyscallError(ret));
            }
        };
        Ok(())
    }

    fn poison(start: GuestAddr, len: usize, val: PoisonType) -> Result<(), LibcHostError> {
        unsafe {
            let ret = syscall(
                Self::SYSCALL_NO,
                HostAction::Poison as usize,
                start,
                len,
                val as usize,
            );
            if ret != 0 {
                return Err(LibcHostError::SyscallError(ret));
            }
        };
        Ok(())
    }

    fn unpoison(start: GuestAddr, len: usize) -> Result<(), LibcHostError> {
        let ret = unsafe { syscall(Self::SYSCALL_NO, HostAction::Unpoison as usize, start, len) };
        if ret != 0 {
            return Err(LibcHostError::SyscallError(ret));
        }
        Ok(())
    }

    fn is_poison(start: GuestAddr, len: usize) -> Result<bool, LibcHostError> {
        let ret = unsafe { syscall(Self::SYSCALL_NO, HostAction::IsPoison as usize, start, len) };
        match ret {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(LibcHostError::SyscallError(ret)),
        }
    }

    fn swap(enabled: bool) -> Result<(), LibcHostError> {
        unsafe {
            let ret = syscall(
                Self::SYSCALL_NO,
                HostAction::SwapState as usize,
                enabled as usize,
            );
            if ret != 0 {
                return Err(LibcHostError::SyscallError(ret));
            }
        };
        Ok(())
    }

    fn alloc(start: GuestAddr, len: usize) -> Result<(), LibcHostError> {
        unsafe {
            let ret = syscall(Self::SYSCALL_NO, HostAction::Alloc as usize, start, len);
            if ret != 0 {
                return Err(LibcHostError::SyscallError(ret));
            }
        };
        Ok(())
    }

    fn dealloc(start: GuestAddr) -> Result<(), LibcHostError> {
        let ret = unsafe { syscall(Self::SYSCALL_NO, HostAction::Dealloc as usize, start) };
        if ret != 0 {
            return Err(LibcHostError::SyscallError(ret));
        }
        Ok(())
    }
}

impl LibcHost {
    const SYSCALL_NO: c_long = 0xa2a4;
}

#[derive(Error, Debug, PartialEq)]
pub enum LibcHostError {
    #[error("Syscall error: {0:?}")]
    SyscallError(c_long),
}
