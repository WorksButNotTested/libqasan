//! # hooks
//!
//! This module provides the implementation of various functions implemented by
//! the standard C library which are used by applications. These functions are
//! are modified to provide the additional memory safety checks provided by
//! `asan`.
pub mod aligned_alloc;
pub mod atoi;
pub mod atol;
pub mod atoll;
pub mod bcmp;
pub mod bzero;
pub mod calloc;
pub mod explicit_bzero;
pub mod free;
pub mod malloc;
pub mod malloc_usable_size;
pub mod memalign;
pub mod memchr;
pub mod memcmp;
pub mod memcpy;
pub mod memmem;
pub mod memmove;
pub mod mempcpy;
pub mod memrchr;
pub mod memset;
pub mod mmap;
pub mod munmap;
pub mod posix_memalign;
pub mod pvalloc;
pub mod read;
pub mod realloc;
pub mod reallocarray;
pub mod stpcpy;
pub mod strcasecmp;
pub mod strcasestr;
pub mod strcat;
pub mod strchr;
pub mod strcmp;
pub mod strcpy;
pub mod strdup;
pub mod strlen;
pub mod strncasecmp;
pub mod strncmp;
pub mod strncpy;
pub mod strndup;
pub mod strnlen;
pub mod strrchr;
pub mod strstr;
pub mod valloc;
pub mod wcscmp;
pub mod wcscpy;
pub mod wcslen;
pub mod write;

#[cfg(feature = "test")]
pub mod test;

#[cfg(feature = "test")]
pub use test::*;

#[cfg(feature = "libc")]
pub mod fgets;

use crate::mmap::Mmap;

use {
    crate::{
        hooks,
        maps::{entry::MapEntry, iterator::MapIterator, MapReader},
        patch::Patch,
        symbols::Symbols,
        GuestAddr,
    },
    alloc::{collections::BTreeMap, vec::Vec},
    core::ffi::{c_char, c_int, c_void, CStr},
    itertools::Itertools,
    log::{debug, trace},
    spin::Mutex,
    thiserror::Error,
};

#[cfg(all(feature = "linux", not(feature = "libc")))]
#[allow(non_camel_case_types)]
pub type size_t = usize;

#[cfg(feature = "libc")]
#[allow(non_camel_case_types)]
pub type size_t = libc::size_t;

#[cfg(all(feature = "linux", not(feature = "libc")))]
#[allow(non_camel_case_types)]
pub type ssize_t = isize;

#[cfg(feature = "libc")]
#[allow(non_camel_case_types)]
pub type ssize_t = libc::ssize_t;

#[cfg(all(feature = "linux", not(feature = "libc")))]
#[allow(non_camel_case_types)]
pub type wchar_t = i32;

#[cfg(feature = "libc")]
#[allow(non_camel_case_types)]
pub type wchar_t = libc::wchar_t;

#[cfg(all(feature = "linux", not(feature = "libc")))]
#[allow(non_camel_case_types)]
pub type off_t = isize;

#[cfg(feature = "libc")]
#[allow(non_camel_case_types)]
pub type off_t = libc::off_t;

#[cfg(not(feature = "test"))]
extern "C" {
    pub fn asan_load(addr: *const c_void, size: usize);
    pub fn asan_store(addr: *const c_void, size: usize);
    pub fn asan_alloc(len: usize, align: usize) -> *mut c_void;
    pub fn asan_dealloc(addr: *const c_void);
    pub fn asan_get_size(addr: *const c_void) -> usize;
    #[cfg(feature = "libc")]
    pub fn asan_sym(name: *const c_char) -> GuestAddr;
    pub fn asan_page_size() -> usize;
    pub fn asan_unpoison(addr: *mut c_void, len: usize);
    pub fn asan_track(addr: *mut c_void, len: usize);
    pub fn asan_untrack(addr: *mut c_void);
    pub fn asan_panic(msg: *const c_char) -> !;
}

extern "C" {
    pub fn asprintf(strp: *mut *mut c_char, fmt: *const c_char, ...) -> c_int;
    pub fn vasprintf(strp: *mut *mut c_char, fmt: *const c_char, va: *const c_void) -> c_int;
}

#[derive(Clone)]
struct PatchFunction {
    pub name: &'static CStr,
    pub destination: GuestAddr,
}

impl PatchFunction {
    const fn new<F: Copy>(name: &'static CStr, func: F) -> Self {
        let pf = (&func) as *const F as *const GuestAddr;
        let destination = unsafe { *pf };
        Self { name, destination }
    }

    fn all() -> Vec<Self> {
        [
            PatchFunction::new::<unsafe extern "C" fn(size_t, size_t) -> *mut c_void>(
                c"aligned_alloc",
                hooks::aligned_alloc::aligned_alloc,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*mut *mut c_char, *const c_char, ...) -> c_int>(
                c"asprintf",
                hooks::asprintf,
            ),
            PatchFunction::new::<
                unsafe extern "C" fn(*const c_void, *const c_void, n: size_t) -> c_int,
            >(c"bcmp", hooks::bcmp::bcmp),
            PatchFunction::new::<unsafe extern "C" fn(*mut c_void, size_t)>(
                c"bzero",
                hooks::bzero::bzero,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*mut c_void, size_t)>(
                c"explicit_bzero",
                hooks::explicit_bzero::explicit_bzero,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_void, c_int, size_t) -> *mut c_void>(
                c"memchr",
                hooks::memchr::memchr,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_void, *const c_void, size_t) -> c_int>(
                c"memcmp",
                hooks::memcmp::memcmp,
            ),
            PatchFunction::new::<
                unsafe extern "C" fn(*mut c_void, *const c_void, size_t) -> *mut c_void,
            >(c"memcpy", hooks::memcpy::memcpy),
            PatchFunction::new::<
                unsafe extern "C" fn(*const c_void, size_t, *const c_void, size_t) -> *mut c_void,
            >(c"memmem", hooks::memmem::memmem),
            PatchFunction::new::<
                unsafe extern "C" fn(*mut c_void, *const c_void, size_t) -> *mut c_void,
            >(c"memmove", hooks::memmove::memmove),
            PatchFunction::new::<
                unsafe extern "C" fn(*mut c_void, *const c_void, size_t) -> *mut c_void,
            >(c"mempcpy", hooks::mempcpy::mempcpy),
            PatchFunction::new::<unsafe extern "C" fn(*const c_void, c_int, size_t) -> *mut c_void>(
                c"memrchr",
                hooks::memrchr::memrchr,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*mut c_char, *const c_char) -> *mut c_char>(
                c"stpcpy",
                hooks::stpcpy::stpcpy,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, *const c_char) -> c_int>(
                c"strcasecmp",
                hooks::strcasecmp::strcasecmp,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char>(
                c"strcasestr",
                hooks::strcasestr::strcasestr,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*mut c_char, *const c_char) -> *mut c_char>(
                c"strcat",
                hooks::strcat::strcat,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, c_int) -> *mut c_char>(
                c"strchr",
                hooks::strchr::strchr,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, *const c_char) -> c_int>(
                c"strcmp",
                hooks::strcmp::strcmp,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*mut c_char, *const c_char) -> *mut c_char>(
                c"strcpy",
                hooks::strcpy::strcpy,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char) -> *mut c_char>(
                c"strdup",
                hooks::strdup::strdup,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char) -> size_t>(
                c"strlen",
                hooks::strlen::strlen,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, *const c_char, size_t) -> c_int>(
                c"strncasecmp",
                hooks::strncasecmp::strncasecmp,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, *const c_char, size_t) -> c_int>(
                c"strncmp",
                hooks::strncmp::strncmp,
            ),
            PatchFunction::new::<
                unsafe extern "C" fn(*mut c_char, *const c_char, size_t) -> *mut c_char,
            >(c"strncpy", hooks::strncpy::strncpy),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, size_t) -> *mut c_char>(
                c"strndup",
                hooks::strndup::strndup,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, size_t) -> size_t>(
                c"strnlen",
                hooks::strnlen::strnlen,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, c_int) -> *mut c_char>(
                c"strrchr",
                hooks::strrchr::strrchr,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char>(
                c"strstr",
                hooks::strstr::strstr,
            ),
            PatchFunction::new::<
                unsafe extern "C" fn(*mut *mut c_char, *const c_char, *const c_void) -> c_int,
            >(c"vasprintf", hooks::vasprintf),
            PatchFunction::new::<unsafe extern "C" fn(*const wchar_t, *const wchar_t) -> c_int>(
                c"wcscmp",
                hooks::wcscmp::wcscmp,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*mut wchar_t, *const wchar_t) -> *mut wchar_t>(
                c"wcscpy",
                hooks::wcscpy::wcscpy,
            ),
            PatchFunction::new::<unsafe extern "C" fn(*const wchar_t) -> size_t>(
                c"wcslen",
                hooks::wcslen::wcslen,
            ),
        ].to_vec()
    }
}

static PATCHED: Mutex<Option<BTreeMap<GuestAddr, PatchFunction>>> = Mutex::new(None);

pub struct Patches;

impl Patches {
    pub fn init<S: Symbols, P: Patch, R: MapReader, M: Mmap>(
    ) -> Result<(), PatchesError<S, P, R, M>> {
        debug!("Installing patches");
        let reader = R::new().map_err(|e| PatchesError::MapReaderError(e))?;
        let mappings = MapIterator::new(reader).collect::<Vec<MapEntry>>();
        mappings.iter().for_each(|m| trace!("{m:?}"));
        let patches = PatchFunction::all()
            .into_iter()
            .map(|p| Self::apply_patch(p, &mappings))
            .collect::<Result<BTreeMap<GuestAddr, PatchFunction>, PatchesError<S, P, R, M>>>()?;
        PATCHED.lock().replace(patches);
        debug!("Patching complete");
        Ok(())
    }

    fn apply_patch<S: Symbols, P: Patch, R: MapReader, M: Mmap>(
        p: PatchFunction,
        mappings: &Vec<MapEntry>,
    ) -> Result<(GuestAddr, PatchFunction), PatchesError<S, P, R, M>> {
        trace!("patch: {:?}, destination: {:#x}", p.name, p.destination);
        let target = S::lookup(p.name.as_ptr() as *const c_char)
            .map_err(|e| PatchesError::SymbolsError(e))?;
        trace!("patching: {:#x} -> {:#x}", target, p.destination);
        let mapping = mappings
            .iter()
            .filter(|m| m.contains(target))
            .exactly_one()
            .map_err(|_e| PatchesError::MapError(target))?;
        let prot = mapping
            .writeable::<M>()
            .map_err(|e| PatchesError::MmapError(e))?;
        P::patch(target, p.destination).map_err(|e| PatchesError::PatchError(e))?;
        drop(prot);
        Ok((target, p))
    }

    pub fn check_patched(addr: GuestAddr) -> Result<(), PatchesCheckError> {
        if let Some(patch) = PATCHED.lock().as_ref().and_then(|p| p.get(&addr)) {
            Err(PatchesCheckError::AddressPatchedError(addr, patch.name))?
        } else {
            Ok(())
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum PatchesError<S: Symbols, P: Patch, R: MapReader, M: Mmap> {
    #[error("Symbols error: {0:?}")]
    SymbolsError(S::Error),
    #[error("Patch error: {0:?}")]
    PatchError(P::Error),
    #[error("Map reader error: {0:?}")]
    MapReaderError(R::Error),
    #[error("Map error: {0:?}")]
    MapError(GuestAddr),
    #[error("Mmap error: {0:?}")]
    MmapError(M::Error),
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum PatchesCheckError {
    #[error("Address: {0} is patched for {1:?}")]
    AddressPatchedError(GuestAddr, &'static CStr),
}
