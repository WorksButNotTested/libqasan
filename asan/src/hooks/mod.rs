pub mod aligned_alloc;
pub mod calloc;
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
