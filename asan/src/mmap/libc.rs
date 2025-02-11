//! # libc
//! This implementation of `Mmap` uses the `libc` crate and hence the standard
//! `libc` library for allocating pages. It should therefore support most
//! operating systems which provide a `libc` library. But is no suited to
//! applications where the library has been statically linked.
use {
    crate::{
        mmap::{Mmap, MmapProt},
        symbols::{
            AtomicGuestAddr, Function, FunctionPointer, FunctionPointerError, Symbols,
            SymbolsLookupStr,
        },
        GuestAddr,
    },
    core::{
        cmp::Ordering,
        ffi::{c_int, c_void},
        marker::PhantomData,
        ptr::null_mut,
        slice::{from_raw_parts, from_raw_parts_mut},
    },
    libc::{
        off_t, size_t, MADV_DONTDUMP, MADV_HUGEPAGE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE,
    },
    log::trace,
    thiserror::Error,
};

#[derive(Debug)]
struct FunctionMmap;

impl Function for FunctionMmap {
    type Func =
        unsafe extern "C" fn(*mut c_void, size_t, c_int, c_int, c_int, off_t) -> *mut c_void;
    const NAME: &'static str = "mmap\0";
}

#[derive(Debug)]
struct FunctionMunmap;

impl Function for FunctionMunmap {
    type Func = unsafe extern "C" fn(*mut c_void, size_t) -> c_int;
    const NAME: &'static str = "munmap\0";
}

#[derive(Debug)]
struct FunctionMprotect;

impl Function for FunctionMprotect {
    type Func = unsafe extern "C" fn(*mut c_void, size_t, c_int) -> c_int;
    const NAME: &'static str = "mprotect\0";
}

#[derive(Debug)]
struct FunctionErrnoLocation;

impl Function for FunctionErrnoLocation {
    type Func = unsafe extern "C" fn() -> *mut c_int;
    const NAME: &'static str = "errno_location\0";
}

#[derive(Debug)]
struct FunctionMadvise;

impl Function for FunctionMadvise {
    type Func = unsafe extern "C" fn(*mut c_void, size_t, c_int) -> c_int;
    const NAME: &'static str = "madvise\0";
}

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

static MMAP_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();
static MUNMAP_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();
static MPROTECT_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();
static GET_ERRNO_LOCATION_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();
static MADVISE_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();

impl<S: Symbols> LibcMmap<S> {
    fn get_mmap() -> Result<<FunctionMmap as Function>::Func, LibcMapError<S>> {
        let addr = MMAP_ADDR.try_get_or_insert_with(|| {
            S::lookup_str(FunctionMmap::NAME).map_err(|e| LibcMapError::FailedToFindSymbol(e))
        })?;
        let f = FunctionMmap::as_ptr(addr).map_err(|e| LibcMapError::InvalidPointerType(e))?;
        Ok(f)
    }

    fn get_munmap() -> Result<<FunctionMunmap as Function>::Func, LibcMapError<S>> {
        let addr = MUNMAP_ADDR.try_get_or_insert_with(|| {
            S::lookup_str(FunctionMunmap::NAME).map_err(|e| LibcMapError::FailedToFindSymbol(e))
        })?;
        let f = FunctionMunmap::as_ptr(addr).map_err(|e| LibcMapError::InvalidPointerType(e))?;
        Ok(f)
    }

    fn get_mprotect() -> Result<<FunctionMprotect as Function>::Func, LibcMapError<S>> {
        let addr = MPROTECT_ADDR.try_get_or_insert_with(|| {
            S::lookup_str(FunctionMprotect::NAME).map_err(|e| LibcMapError::FailedToFindSymbol(e))
        })?;
        let f = FunctionMprotect::as_ptr(addr).map_err(|e| LibcMapError::InvalidPointerType(e))?;
        Ok(f)
    }

    fn get_errno_location() -> Result<<FunctionErrnoLocation as Function>::Func, LibcMapError<S>> {
        let addr = GET_ERRNO_LOCATION_ADDR.try_get_or_insert_with(|| {
            S::lookup_str(FunctionErrnoLocation::NAME)
                .map_err(|e| LibcMapError::FailedToFindSymbol(e))
        })?;
        let f =
            FunctionErrnoLocation::as_ptr(addr).map_err(|e| LibcMapError::InvalidPointerType(e))?;
        Ok(f)
    }

    fn get_madvise() -> Result<<FunctionMadvise as Function>::Func, LibcMapError<S>> {
        let addr = MADVISE_ADDR.try_get_or_insert_with(|| {
            S::lookup_str(FunctionMadvise::NAME).map_err(|e| LibcMapError::FailedToFindSymbol(e))
        })?;
        let f = FunctionMadvise::as_ptr(addr).map_err(|e| LibcMapError::InvalidPointerType(e))?;
        Ok(f)
    }

    fn errno() -> Result<c_int, LibcMapError<S>> {
        let errno_location = Self::get_errno_location()?;
        let errno = unsafe { *errno_location() };
        Ok(errno)
    }
}

impl<S: Symbols> Mmap for LibcMmap<S> {
    type Error = LibcMapError<S>;

    fn map(len: usize) -> Result<LibcMmap<S>, LibcMapError<S>> {
        let mmap = Self::get_mmap()?;
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
            let errno = Self::errno()?;
            Err(LibcMapError::FailedToMap(len, errno))
        } else {
            let addr = map as GuestAddr;
            Ok(LibcMmap {
                addr,
                len,
                _phantom: PhantomData,
            })
        }
    }

    fn map_at(addr: GuestAddr, len: usize) -> Result<LibcMmap<S>, LibcMapError<S>> {
        let mmap = Self::get_mmap()?;
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
            let errno = Self::errno()?;
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
        let mprotect = Self::get_mprotect()?;
        let ret = unsafe { mprotect(addr as *mut c_void, len, c_int::from(&prot)) };
        if ret != 0 {
            let errno = Self::errno()?;
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

    fn huge_pages(addr: GuestAddr, len: usize) -> Result<(), Self::Error> {
        trace!("huge_pages - addr: {:#x}, len: {:#x}", addr, len);
        let madvise = Self::get_madvise()?;
        let ret = unsafe { madvise(addr as *mut c_void, len, MADV_HUGEPAGE) };
        if ret != 0 {
            let errno = Self::errno()?;
            Err(LibcMapError::FailedToMadviseHugePage(addr, len, errno))?;
        }
        Ok(())
    }

    fn dont_dump(addr: GuestAddr, len: usize) -> Result<(), Self::Error> {
        trace!("dont_dump - addr: {:#x}, len: {:#x}", addr, len);
        let madvise = Self::get_madvise()?;
        let ret = unsafe { madvise(addr as *mut c_void, len, MADV_DONTDUMP) };
        if ret != 0 {
            let errno = Self::errno()?;
            Err(LibcMapError::FailedToMadviseDontDump(addr, len, errno))?;
        }
        Ok(())
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
            let munmap = Self::get_munmap().unwrap();
            if munmap(self.addr as *mut c_void, self.len) < 0 {
                let errno = Self::errno().unwrap();
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
    FailedToFindSymbol(S::Error),
    #[error("Failed to mprotect - addr: {0}, len: {1}, prot: {2:?}, errno: {3}")]
    FailedToMprotect(GuestAddr, usize, MmapProt, c_int),
    #[error("Invalid pointer type: {0:?}")]
    InvalidPointerType(FunctionPointerError),
    #[error("Failed to madvise HUGEPAGE - addr: {0}, len: {1}, errno: {2}")]
    FailedToMadviseHugePage(GuestAddr, usize, c_int),
    #[error("Failed to madvise DONTDUMP - addr: {0}, len: {1}, errno: {2}")]
    FailedToMadviseDontDump(GuestAddr, usize, c_int),
}
