//! # libc
//! This implementation of `Mmap` uses the `libc` crate and hence the standard
//! `libc` library for allocating pages. It should therefore support most
//! operating systems which provide a `libc` library. But is no suited to
//! applications where the library has been statically linked.
use {
    crate::{
        mmap::{Mmap, MmapProt},
        symbols::Symbols,
        GuestAddr,
    },
    core::{
        cmp::Ordering,
        ffi::{c_int, c_void},
        marker::PhantomData,
        ptr::null_mut,
        slice::{from_raw_parts, from_raw_parts_mut},
    },
    libc::{__errno_location, off_t, size_t, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE},
    log::trace,
    spin::Mutex,
    thiserror::Error,
};

type FnMmap = unsafe extern "C" fn(*mut c_void, size_t, c_int, c_int, c_int, off_t) -> *mut c_void;
type FnMunmap = unsafe extern "C" fn(*mut c_void, size_t) -> c_int;
type FnMprotect = unsafe extern "C" fn(*mut c_void, size_t, c_int) -> c_int;

static MMAP_FUNC: Mutex<Option<FnMmap>> = Mutex::new(None);
static MUNMAP_FUNC: Mutex<Option<FnMunmap>> = Mutex::new(None);
static MPROTECT_FUNC: Mutex<Option<FnMprotect>> = Mutex::new(None);

#[derive(Debug)]
pub struct LibcMmap<S: Symbols> {
    addr: GuestAddr,
    len: usize,
    _phantom: PhantomData<S>,
}

impl<S: Symbols> Ord for LibcMmap<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl<S: Symbols> PartialOrd for LibcMmap<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Symbols> PartialEq for LibcMmap<S> {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl<S: Symbols> Eq for LibcMmap<S> {}

impl<S: Symbols> LibcMmap<S> {
    fn get_mmap() -> Result<FnMmap, S::Error> {
        Ok(*MMAP_FUNC.lock().get_or_insert(S::lookup("mmap")?))
    }

    fn get_munmap() -> Result<FnMunmap, S::Error> {
        Ok(*MUNMAP_FUNC.lock().get_or_insert(S::lookup("munmap")?))
    }

    fn get_mprotect() -> Result<FnMprotect, S::Error> {
        Ok(*MPROTECT_FUNC.lock().get_or_insert(S::lookup("mprotect")?))
    }
}

impl<S: Symbols> Mmap for LibcMmap<S> {
    type Error = LibcMapError<S>;

    fn map(len: usize) -> Result<LibcMmap<S>, LibcMapError<S>> {
        let mmap = Self::get_mmap().map_err(|e| LibcMapError::<S>::FailedToFindMmapFunctions(e))?;
        let map = unsafe {
            mmap(
                null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };
        if map == libc::MAP_FAILED {
            let errno = unsafe { *__errno_location() };
            Err(LibcMapError::FailedToMap(len, errno))
        } else {
            let addr = map as GuestAddr;
            trace!("Mapped: 0x{:x}-0x{:x}", addr, addr + len);
            Ok(LibcMmap {
                addr,
                len,
                _phantom: PhantomData,
            })
        }
    }

    fn map_at(addr: GuestAddr, len: usize) -> Result<LibcMmap<S>, LibcMapError<S>> {
        let mmap = Self::get_mmap().map_err(|e| LibcMapError::<S>::FailedToFindMmapFunctions(e))?;
        let map = unsafe {
            mmap(
                addr as *mut c_void,
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE
                    | libc::MAP_ANONYMOUS
                    | libc::MAP_NORESERVE
                    | libc::MAP_FIXED
                    | libc::MAP_FIXED_NOREPLACE,
                -1,
                0,
            )
        };
        trace!("Mapped: 0x{:x}-0x{:x}", addr, addr + len);
        if map == libc::MAP_FAILED {
            let errno = unsafe { *__errno_location() };
            Err(LibcMapError::FailedToMapAt(addr, len, errno))
        } else {
            Ok(LibcMmap {
                addr,
                len,
                _phantom: PhantomData,
            })
        }
    }

    fn protect(addr: GuestAddr, len: usize, prot: MmapProt) -> Result<(), Self::Error> {
        trace!(
            "protect - addr: {:#x}, len: {:#x}, prot: {:#x}",
            addr,
            len,
            prot
        );
        let mprotect =
            Self::get_mprotect().map_err(|e| LibcMapError::<S>::FailedToFindMmapFunctions(e))?;
        let ret = unsafe { mprotect(addr as *mut c_void, len, c_int::from(&prot)) };
        if ret != 0 {
            let errno = unsafe { *__errno_location() };
            Err(LibcMapError::FailedToMprotect(addr, len, prot, errno))?;
        }

        Ok(())
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.addr as *const u8, self.len) }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.addr as *mut u8, self.len) }
    }
}

impl From<&MmapProt> for c_int {
    fn from(prot: &MmapProt) -> Self {
        let mut ret = PROT_NONE;
        if prot.contains(MmapProt::READ) {
            ret |= PROT_READ;
        }
        if prot.contains(MmapProt::WRITE) {
            ret |= PROT_WRITE;
        }
        if prot.contains(MmapProt::EXEC) {
            ret |= PROT_EXEC;
        }
        ret
    }
}

impl<S: Symbols> Drop for LibcMmap<S> {
    fn drop(&mut self) {
        unsafe {
            let munmap = Self::get_munmap()
                .map_err(|e| LibcMapError::<S>::FailedToFindMmapFunctions(e))
                .unwrap();
            if munmap(self.addr as *mut c_void, self.len) < 0 {
                let errno = *__errno_location();
                panic!("Errno: {}", errno);
            }
            trace!("Unmapped: 0x{:x}-0x{:x}", self.addr, self.addr + self.len);
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum LibcMapError<S: Symbols> {
    #[error("Failed to map - len: {0}, errno: {1}")]
    FailedToMap(usize, c_int),
    #[error("Failed to map: {0}, len: {1}, errno: {2}")]
    FailedToMapAt(GuestAddr, usize, c_int),
    #[error("Failed to find mmap functions")]
    FailedToFindMmapFunctions(S::Error),
    #[error("Failed to mprotect - addr: {0}, len: {1}, prot: {2:?}, errno: {3}")]
    FailedToMprotect(GuestAddr, usize, MmapProt, c_int),
}
