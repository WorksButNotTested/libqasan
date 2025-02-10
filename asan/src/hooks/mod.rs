pub mod aligned_alloc;
pub mod calloc;
pub mod free;
pub mod malloc;
pub mod malloc_usable_size;
pub mod memalign;
pub mod posix_memalign;
pub mod pvalloc;
pub mod read;
pub mod realloc;
pub mod reallocarray;
pub mod valloc;
pub mod write;

use core::ffi::c_void;
#[cfg(feature = "libc")]
use {crate::GuestAddr, core::ffi::c_char};

#[cfg(all(feature = "linux", not(feature = "libc")))]
#[allow(non_camel_case_types)]
type size_t = usize;

#[cfg(feature = "libc")]
#[allow(non_camel_case_types)]
type size_t = libc::size_t;

#[cfg(all(feature = "linux", not(feature = "libc")))]
#[allow(non_camel_case_types)]
type ssize_t = isize;

#[cfg(feature = "libc")]
#[allow(non_camel_case_types)]
type ssize_t = libc::ssize_t;

extern "C" {
    fn asan_load(addr: *const c_void, size: usize);
    fn asan_store(addr: *const c_void, size: usize);
    fn asan_alloc(len: usize, align: usize) -> *mut c_void;
    fn asan_dealloc(addr: *const c_void);
    fn asan_get_size(addr: *const c_void) -> usize;
    #[cfg(feature = "libc")]
    fn asan_sym(name: *const c_char) -> GuestAddr;
    fn asan_page_size() -> usize;
}
