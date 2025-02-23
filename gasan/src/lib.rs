#![cfg_attr(not(feature = "test"), no_std)]
extern crate alloc;

use {
    asan::{
        allocator::{
            backend::{dlmalloc::DlmallocBackend, mimalloc::MimallocBackend, GlobalAllocator},
            frontend::{default::DefaultFrontend, AllocatorFrontend},
        },
        logger::libc::LibcLogger,
        maps::libc::LibcMapReader,
        mmap::libc::LibcMmap,
        patch::{hooks::PatchedHooks, raw::RawPatch},
        shadow::{
            guest::{DefaultShadowLayout, GuestShadow},
            Shadow,
        },
        symbols::{
            dlsym::{DlSymSymbols, LookupTypeNext},
            Symbols,
        },
        tracking::{guest::GuestTracking, Tracking},
        GuestAddr,
    },
    core::ffi::{c_char, c_void, CStr},
    ctor::ctor,
    log::{info, trace, Level},
    spin::{mutex::Mutex, Lazy},
};

type Syms = DlSymSymbols<LookupTypeNext>;

type GasanMmap = LibcMmap<Syms>;

type GasanBackend = MimallocBackend<GlobalAllocator<DlmallocBackend<GasanMmap>>>;

pub type GasanFrontend =
    DefaultFrontend<GasanBackend, GuestShadow<GasanMmap, DefaultShadowLayout>, GuestTracking>;

pub type GasanSyms = DlSymSymbols<LookupTypeNext>;

const PAGE_SIZE: usize = 4096;

static FRONTEND: Lazy<Mutex<GasanFrontend>> = Lazy::new(|| {
    LibcLogger::initialize::<GasanSyms>(Level::Trace);
    info!("init");
    let backend = GasanBackend::new(GlobalAllocator::new(DlmallocBackend::new(PAGE_SIZE)));
    let shadow = GuestShadow::<GasanMmap, DefaultShadowLayout>::new().unwrap();
    let tracking = GuestTracking::new().unwrap();
    let frontend = GasanFrontend::new(
        backend,
        shadow,
        tracking,
        GasanFrontend::DEFAULT_REDZONE_SIZE,
        GasanFrontend::DEFAULT_QUARANTINE_SIZE,
    )
    .unwrap();
    PatchedHooks::init::<GasanSyms, RawPatch, LibcMapReader<GasanSyms>, GasanMmap>().unwrap();
    Mutex::new(frontend)
});

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_load(addr: *const c_void, size: usize) {
    trace!("load - addr: 0x{:x}, size: {:#x}", addr as GuestAddr, size);
    if FRONTEND
        .lock()
        .shadow()
        .is_poison(addr as GuestAddr, size)
        .unwrap()
    {
        panic!("Poisoned - addr: {:p}, size: 0x{:x}", addr, size);
    }
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_store(addr: *const c_void, size: usize) {
    trace!("store - addr: 0x{:x}, size: {:#x}", addr as GuestAddr, size);
    if FRONTEND
        .lock()
        .shadow()
        .is_poison(addr as GuestAddr, size)
        .unwrap()
    {
        panic!("Poisoned - addr: {:p}, size: 0x{:x}", addr, size);
    }
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_alloc(len: usize, align: usize) -> *mut c_void {
    trace!("alloc - len: {:#x}, align: {:#x}", len, align);
    let ptr = FRONTEND.lock().alloc(len, align).unwrap() as *mut c_void;
    trace!(
        "alloc - len: {:#x}, align: {:#x}, ptr: {:p}",
        len,
        align,
        ptr
    );
    ptr
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_dealloc(addr: *const c_void) {
    trace!("free - addr: {:p}", addr);
    FRONTEND.lock().dealloc(addr as GuestAddr).unwrap();
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_get_size(addr: *const c_void) -> usize {
    trace!("get_size - addr: {:p}", addr);
    FRONTEND.lock().get_size(addr as GuestAddr).unwrap()
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_sym(name: *const c_char) -> GuestAddr {
    GasanSyms::lookup(name).unwrap()
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_page_size() -> usize {
    PAGE_SIZE
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_unpoison(addr: *const c_void, len: usize) {
    trace!("unpoison - addr: {:p}, len: {:#x}", addr, len);
    FRONTEND
        .lock()
        .shadow_mut()
        .unpoison(addr as GuestAddr, len)
        .unwrap();
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_track(addr: *const c_void, len: usize) {
    trace!("track - addr: {:p}, len: {:#x}", addr, len);
    FRONTEND
        .lock()
        .tracking_mut()
        .alloc(addr as GuestAddr, len)
        .unwrap();
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_untrack(addr: *const c_void) {
    trace!("untrack - addr: {:p}", addr);
    FRONTEND
        .lock()
        .tracking_mut()
        .dealloc(addr as GuestAddr)
        .unwrap();
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_panic(msg: *const c_char) -> ! {
    trace!("panic - msg: {:p}", msg);
    let msg = unsafe { CStr::from_ptr(msg as *const c_char) };
    panic!("{}", msg.to_str().unwrap());
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn asan_swap(_enabled: bool) {
    /* Don't log since this function is on the logging path */
}

#[no_mangle]
#[ctor]
fn ctor() {
    drop(FRONTEND.lock());
}
