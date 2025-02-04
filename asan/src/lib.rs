//! # asan
//!
//! `asan` is a library intended to be used by a guest running in QEMU to
//! support address sanitizer.
//!
//! It has a modular design intended to support different use cases and
//! environments. The following initial variants are proposed:
//!
//! - `qasan` - Intended as a drop in replacement for the original libqasan,
//!   this will have dependency on `libc` and will interact with QEMU using the
//!   bespoke syscall interface to perform memory tracking and shadowing.
//! - `gasan` - This is similar to `qasan`, but rather than having QEMU perform
//!   the management of the shadow memory and memory tracking, this work will be
//!   carried out purely in the guest (and hence should be more performant).
//! - `zasan` - This variant is intended to have no dependencies on libc, nor
//!   any other libraries. It is intended to be used in bare-metal targets or
//!   targets which have statically linked `libc`.
//!
//! The componentized nature of the design is intended to permit the user to
//! adapt `asan` to their needs with minimal modification by selecting and
//! combining alternative implementations of the various key components.
//!
//! ## Features
//! - `dlmalloc` - Enable support for the dlmalloc allocator backend.
//! - `guest` - Enable support for shadow memory and tracking in the guest
//! - `host` - Enable support for shadow memory and tracking in the host
//! - `libc` - Enable use of `LibcMmap` to support creation of mappings using
//!   `libc`
//! - `linux` - Enable use of `LinuxMmap` to support creation of mappings and
//!   host interaction using `rustix`.
//! - `std` - Disable the magic used to support `no_std` environments
//!
#![cfg_attr(not(feature = "test"), no_std)]
#![cfg_attr(target_arch = "powerpc", feature(asm_experimental_arch))]

use {alloc::boxed::Box, core::ffi::c_void, spin::Mutex, symbols::Symbol};

pub mod allocator;

#[cfg(not(feature = "test"))]
pub mod arch;

#[cfg(feature = "hooks")]
pub mod hooks;

#[cfg(feature = "host")]
pub mod host;

pub mod logger;

#[cfg(not(feature = "test"))]
pub mod mem;

pub mod mmap;

#[cfg(not(feature = "test"))]
mod nostd;

pub mod patch;

pub mod shadow;

pub mod symbols;

pub mod tracking;

extern crate alloc;

pub type GuestAddr = usize;

static ASAN: Mutex<Option<Box<dyn Asan>>> = Mutex::new(None);

pub trait Asan: Send {
    fn asan_load(&mut self, addr: *const c_void, size: usize);
    fn asan_store(&mut self, addr: *const c_void, size: usize);
    fn asan_alloc(&mut self, len: usize, align: usize) -> *mut c_void;
    fn asan_dealloc(&mut self, addr: *const c_void);
    fn asan_get_size(&mut self, addr: *const c_void) -> usize;
    fn asan_sym(&mut self, name: &'static str) -> Symbol;
    fn asan_page_size(&self) -> usize;
}

pub fn asan_init<A: Asan + 'static>(asan: A) {
    ASAN.lock().replace(Box::new(asan));
}

pub fn asan_load(addr: *const c_void, size: usize) {
    ASAN.lock().as_mut().unwrap().asan_load(addr, size);
}

pub fn asan_store(addr: *const c_void, size: usize) {
    ASAN.lock().as_mut().unwrap().asan_store(addr, size);
}

pub fn asan_alloc(len: usize, align: usize) -> *mut c_void {
    ASAN.lock().as_mut().unwrap().asan_alloc(len, align)
}

pub fn asan_dealloc(addr: *const c_void) {
    ASAN.lock().as_mut().unwrap().asan_dealloc(addr);
}

pub fn asan_get_size(addr: *const c_void) -> usize {
    ASAN.lock().as_mut().unwrap().asan_get_size(addr)
}

pub fn asan_sym(name: &'static str) -> Symbol {
    ASAN.lock().as_mut().unwrap().asan_sym(name)
}

pub fn asan_page_size() -> usize {
    ASAN.lock().as_mut().unwrap().asan_page_size()
}
