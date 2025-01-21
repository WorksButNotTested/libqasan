//! # libc
//! This implementation of `Mmap` uses the `libc` crate and hence the standard
//! `libc` library for allocating pages. It should therefore support most
//! operating systems which provide a `libc` library. But is no suited to
//! applications where the library has been statically linked.
use {
    crate::{mmap::Mmap, GuestAddr},
    alloc::ffi::{CString, NulError},
    core::{
        ffi::{c_int, c_void, CStr},
        mem::transmute,
        ptr::null_mut,
        slice::{from_raw_parts, from_raw_parts_mut},
    },
    libc::{__errno_location, dlerror, dlsym, off_t, size_t, RTLD_NEXT},
    log::trace,
    spin::Lazy,
    thiserror::Error,
};

type FnMmap = unsafe extern "C" fn(*mut c_void, size_t, c_int, c_int, c_int, off_t) -> *mut c_void;
type FnMunmap = unsafe extern "C" fn(*mut c_void, size_t) -> c_int;
const UNKNOWN_ERROR: &str = "Unknown error";

#[derive(Clone)]
struct LibcMmapFuncs {
    mmap: FnMmap,
    munmap: FnMunmap,
}

impl LibcMmapFuncs {
    fn new() -> Result<LibcMmapFuncs, LibcMmapFuncsError> {
        let mmap_cstring = CString::new("mmap")?;
        let mmap_cstr: &CStr = mmap_cstring.as_c_str();
        let p_mmap = unsafe { dlsym(RTLD_NEXT, mmap_cstr.as_ptr()) };
        if p_mmap.is_null() {
            Err(LibcMmapFuncsError::FailedToFindFunction(
                "mmap",
                Self::get_error(),
            ))?;
        }
        let mmap = unsafe { transmute::<*mut c_void, FnMmap>(p_mmap) };

        let munmap_cstring = CString::new("munmap")?;
        let munmap_cstr: &CStr = munmap_cstring.as_c_str();
        let p_munmap = unsafe { dlsym(RTLD_NEXT, munmap_cstr.as_ptr()) };
        if p_munmap.is_null() {
            Err(LibcMmapFuncsError::FailedToFindFunction(
                "munmap",
                Self::get_error(),
            ))?;
        }
        let munmap = unsafe { transmute::<*mut c_void, FnMunmap>(p_munmap) };

        Ok(LibcMmapFuncs { mmap, munmap })
    }

    fn get_error() -> &'static str {
        let error = unsafe { dlerror() };
        if error.is_null() {
            UNKNOWN_ERROR
        } else {
            unsafe { CStr::from_ptr(error).to_str().unwrap_or(UNKNOWN_ERROR) }
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum LibcMmapFuncsError {
    #[error("Bad function name: {0:?}")]
    BadFunctionName(#[from] NulError),
    #[error("Failed to find function: {0}, error: {1}")]
    FailedToFindFunction(&'static str, &'static str),
}

static MMAP_FUNCS: Lazy<Result<LibcMmapFuncs, LibcMmapFuncsError>> = Lazy::new(LibcMmapFuncs::new);

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
pub struct LibcMmap {
    addr: GuestAddr,
    len: usize,
}

impl Mmap for LibcMmap {
    type Error = LibcMapError;

    fn map(len: usize) -> Result<LibcMmap, LibcMapError> {
        let funcs = MMAP_FUNCS
            .as_ref()
            .map_err(|e| LibcMapError::FailedToFindMmapFunctions(e.clone()))?;
        let map = unsafe {
            (funcs.mmap)(
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
            Ok(LibcMmap { addr, len })
        }
    }

    fn map_at(addr: GuestAddr, len: usize) -> Result<LibcMmap, LibcMapError> {
        let funcs = MMAP_FUNCS
            .as_ref()
            .map_err(|e| LibcMapError::FailedToFindMmapFunctions(e.clone()))?;
        let map = unsafe {
            (funcs.mmap)(
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
            Ok(LibcMmap { addr, len })
        }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.addr as *const u8, self.len) }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.addr as *mut u8, self.len) }
    }
}

impl Drop for LibcMmap {
    fn drop(&mut self) {
        unsafe {
            let funcs = MMAP_FUNCS.as_ref().unwrap();
            if (funcs.munmap)(self.addr as *mut c_void, self.len) < 0 {
                let errno = *__errno_location();
                panic!("Errno: {}", errno);
            }
            trace!("Unmapped: 0x{:x}-0x{:x}", self.addr, self.addr + self.len);
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum LibcMapError {
    #[error("Failed to map - len: {0}, errno: {1}")]
    FailedToMap(usize, c_int),
    #[error("Failed to map: {0}, len: {1}, errno: {2}")]
    FailedToMapAt(GuestAddr, usize, c_int),
    #[error("Failed to find mmap functions")]
    FailedToFindMmapFunctions(LibcMmapFuncsError),
}
