#![no_std]
extern crate alloc;

use {
    asan::{
        allocator::frontend::{default::DefaultFrontend, AllocatorFrontend},
        logger::libc::LibcLogger,
        mmap::libc::LibcMmap,
        shadow::{
            guest::{DefaultShadowLayout, GuestShadow},
            Shadow,
        },
        symbols::{
            dlsym::{DlSymSymbols, LookupTypeNext},
            Symbols,
        },
        tracking::guest::GuestTracking,
        GuestAddr,
    },
    core::ffi::{c_char, c_void},
    log::{trace, Level},
    spin::{mutex::Mutex, Lazy},
};

type Syms = DlSymSymbols<LookupTypeNext>;

type BE = asan::allocator::backend::dlmalloc::DlmallocBackend<LibcMmap<Syms>>;

pub type GasanAllocator =
    DefaultFrontend<BE, GuestShadow<LibcMmap<Syms>, DefaultShadowLayout>, GuestTracking>;

pub type GasanSyms = DlSymSymbols<LookupTypeNext>;

const PAGE_SIZE: usize = 4096;

static FRONTEND: Lazy<Mutex<GasanAllocator>> = Lazy::new(|| {
    LibcLogger::initialize::<GasanSyms>(Level::Trace);
    trace!("init");
    let backend = BE::new(PAGE_SIZE);
    let shadow = GuestShadow::<LibcMmap<Syms>, DefaultShadowLayout>::new().unwrap();
    let tracking = GuestTracking::new().unwrap();
    let allocator = GasanAllocator::new(
        backend,
        shadow,
        tracking,
        GasanAllocator::DEFAULT_REDZONE_SIZE,
        GasanAllocator::DEFAULT_QUARANTINE_SIZE,
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
    GasanSyms::lookup(name).unwrap()
}

#[no_mangle]
pub fn asan_page_size() -> usize {
    PAGE_SIZE
}

#[no_mangle]
pub extern "C" fn gasan_allocate(size: usize, align: usize) -> *mut c_void {
    asan_alloc(size, align)
}

#[no_mangle]
pub extern "C" fn gasan_deallocate(addr: *mut c_void) {
    asan_dealloc(addr);
}
