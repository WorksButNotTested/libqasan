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
//! The following ancilliary crates are provided as follows:
//! - `dummy_libc` - A dummy libc library used during linking which provides
//!   only the symbols `dlsym` and `dlerror`. This is intended to ensure that
//!   `gasan` and `qasan` do not have any direct dependency on libc and hence
//!   avoids the possibility of accidental re-entrancy. (e.g. in the case that
//!   we have hooked a function such as `malloc` and in our handling of the call
//!   inadvertently call `malloc`, or one of our other hooked functions
//!   ourselves).
//! - `fuzz` - A few different fuzzing harnesses used to test `asan`.
//!
//! The componentized nature of the design is intended to permit the user to
//! adapt `asan` to their needs with minimal modification by selecting and
//! combining alternative implementations of the various key components.
//!
//! ## Features
//! - `dlmalloc` - Enable support for the dlmalloc allocator backend.
//! - `guest` - Enable support for shadow memory and tracking in the guest
//! - `hooks` - Enable support for hooking functions in the guest
//! - `host` - Enable support for shadow memory and tracking in the host
//! - `libc` - Enable use of the `libc` library to support creation of mappings,
//!   read/write, logging etc (more OS agnostic)
//! - `linux` - Enable the use of direct syscalls (supported by `rustix`) to
//!   interact with the operating system (Linux specific).
//! - `test` - Disable the magic used to support `no_std` environments for
//!   running unit and integration tests
//! - `tracking` - Enable support for memory tracking.
//!
#![cfg_attr(not(feature = "test"), no_std)]
#![cfg_attr(target_arch = "powerpc", feature(asm_experimental_arch))]

pub mod allocator;

#[cfg(not(feature = "test"))]
pub mod arch;

pub mod exit;

#[cfg(feature = "hooks")]
pub mod hooks;

#[cfg(feature = "host")]
pub mod host;

pub mod logger;

pub mod maps;

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
