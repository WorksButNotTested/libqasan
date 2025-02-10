#![no_std]
extern crate alloc;

use {
    asan::{
        allocator::{
            backend::dlmalloc::DlmallocBackend,
            frontend::{default::DefaultFrontend, AllocatorFrontend},
        },
        logger::linux::LinuxLogger,
        mmap::linux::LinuxMmap,
        shadow::{
            guest::{DefaultShadowLayout, GuestShadow},
            Shadow,
        },
        symbols::{nop::NopSymbols, Symbols},
        tracking::guest::GuestTracking,
        GuestAddr,
    },
    core::ffi::{c_char, c_void},
    log::{trace, Level},
    spin::{Lazy, Mutex},
};

pub type ZasanAllocator = DefaultFrontend<
    DlmallocBackend<LinuxMmap>,
    GuestShadow<LinuxMmap, DefaultShadowLayout>,
    GuestTracking,
>;

pub type ZasanSyms = NopSymbols;

const PAGE_SIZE: usize = 4096;

static FRONTEND: Lazy<Mutex<ZasanAllocator>> = Lazy::new(|| {
    LinuxLogger::initialize(Level::Info);
    let backend = DlmallocBackend::<LinuxMmap>::new(PAGE_SIZE);
    let shadow = GuestShadow::<LinuxMmap, DefaultShadowLayout>::new().unwrap();
    let tracking = GuestTracking::new().unwrap();
    let allocator = ZasanAllocator::new(
        backend,
        shadow,
        tracking,
        ZasanAllocator::DEFAULT_REDZONE_SIZE,
        ZasanAllocator::DEFAULT_QUARANTINE_SIZE,
    )
    .unwrap();
    Mutex::new(allocator)
});

#[no_mangle]
pub fn asan_load(addr: *const c_void, size: usize) {
    trace!("load - addr: 0x{:x}, size: {:#x}", addr as GuestAddr, size);
    if FRONTEND
        .lock()
        .shadow()
        .is_poison(addr as GuestAddr, size)
        .unwrap()
    {
        panic!("Poisoned - addr: 0x{:p}, size: 0x{:x}", addr, size);
    }
}

#[no_mangle]
pub fn asan_store(addr: *const c_void, size: usize) {
    trace!("store - addr: 0x{:x}, size: {:#x}", addr as GuestAddr, size);
    if FRONTEND
        .lock()
        .shadow()
        .is_poison(addr as GuestAddr, size)
        .unwrap()
    {
        panic!("Poisoned - addr: 0x{:p}, size: 0x{:x}", addr, size);
    }
}

#[no_mangle]
pub fn asan_alloc(len: usize, align: usize) -> *mut c_void {
    trace!("alloc - len: {:#x}, align: {:#x}", len, align);
    let ptr = FRONTEND.lock().alloc(len, align).unwrap() as *mut c_void;
    trace!(
        "alloc - len: {:#x}, align: {:#x}, ptr: 0x{:p}",
        len,
        align,
        ptr
    );
    ptr
}

#[no_mangle]
pub fn asan_dealloc(addr: *const c_void) {
    trace!("free - addr: 0x{:p}", addr);
    FRONTEND.lock().dealloc(addr as GuestAddr).unwrap();
}

#[no_mangle]
pub fn asan_get_size(addr: *const c_void) -> usize {
    trace!("get_size - addr: 0x{:p}", addr);
    FRONTEND.lock().get_size(addr as GuestAddr).unwrap()
}

#[no_mangle]
pub fn asan_sym(name: *const c_char) -> GuestAddr {
    ZasanSyms::lookup(name).unwrap()
}

#[no_mangle]
pub fn asan_page_size() -> usize {
    PAGE_SIZE
}

#[no_mangle]
pub extern "C" fn zasan_allocate(size: usize, align: usize) -> *mut c_void {
    asan_alloc(size, align)
}

#[no_mangle]
pub extern "C" fn zasan_deallocate(addr: *mut c_void) {
    asan_dealloc(addr);
}
