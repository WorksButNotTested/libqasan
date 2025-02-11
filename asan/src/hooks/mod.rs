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

#[cfg(not(feature = "test"))]
use core::ffi::{c_char, c_void};

#[cfg(all(feature = "libc", not(feature = "test")))]
use crate::GuestAddr;

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
