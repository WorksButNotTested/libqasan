//! # linux
//! The `LinuxHost` supports the established means of interacting with the QEMU
//! emulator on Linux by means of issuing a bespoke syscall.
use {
    crate::{
        host::{Host, HostAction},
        shadow::PoisonType,
        GuestAddr,
    },
    core::mem::transmute,
    syscalls::{syscall2, syscall3, syscall4, Errno, Sysno},
};

#[derive(Debug)]
pub struct LinuxHost;

pub type LinuxHostResult<T> = Result<T, Errno>;

impl Host for LinuxHost {
    type Error = Errno;

    fn load(start: GuestAddr, len: usize) -> LinuxHostResult<()> {
        unsafe {
            syscall3(Self::sysno(), HostAction::CheckLoad as usize, start, len)?;
        }
        Ok(())
    }

    fn store(start: GuestAddr, len: usize) -> LinuxHostResult<()> {
        unsafe {
            syscall3(Self::sysno(), HostAction::CheckStore as usize, start, len)?;
        };
        Ok(())
    }

    fn poison(start: GuestAddr, len: usize, val: PoisonType) -> LinuxHostResult<()> {
        unsafe {
            syscall4(
                Self::sysno(),
                HostAction::Poison as usize,
                start,
                len,
                val as usize,
            )?;
        };
        Ok(())
    }

    fn unpoison(start: GuestAddr, len: usize) -> LinuxHostResult<()> {
        unsafe { syscall3(Self::sysno(), HostAction::Unpoison as usize, start, len)? };
        Ok(())
    }

    fn is_poison(start: GuestAddr, len: usize) -> LinuxHostResult<bool> {
        unsafe { Ok(syscall3(Self::sysno(), HostAction::IsPoison as usize, start, len)? != 0) }
    }

    fn swap(enabled: bool) -> LinuxHostResult<()> {
        unsafe {
            syscall2(
                Self::sysno(),
                HostAction::SwapState as usize,
                enabled as usize,
            )?;
        };
        Ok(())
    }

    fn alloc(start: GuestAddr, len: usize) -> LinuxHostResult<()> {
        unsafe {
            syscall3(Self::sysno(), HostAction::Alloc as usize, start, len)?;
        };
        Ok(())
    }

    fn dealloc(start: GuestAddr) -> LinuxHostResult<()> {
        unsafe { syscall2(Self::sysno(), HostAction::Dealloc as usize, start)? };
        Ok(())
    }
}

impl LinuxHost {
    const SYSCALL_NO: u32 = 0xa2a4;

    pub fn sysno() -> Sysno {
        unsafe { transmute(Self::SYSCALL_NO) }
    }
}
