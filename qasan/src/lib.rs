#![no_std]
extern crate alloc;

use {
    asan::{
        allocator::frontend::{default::DefaultFrontend, AllocatorFrontend},
        host::libc::LibcHost,
        logger::libc::LibcLogger,
        shadow::{host::HostShadow, Shadow},
        symbols::{
            dlsym::{DlSymSymbols, LookupTypeNext},
            Symbols,
        },
        tracking::{host::HostTracking, Tracking},
        GuestAddr,
    },
    core::ffi::{c_char, c_void, CStr},
    log::{trace, Level},
    spin::{Lazy, Mutex},
};

type Syms = DlSymSymbols<LookupTypeNext>;

type BE = asan::allocator::backend::dlmalloc::DlmallocBackend<asan::mmap::libc::LibcMmap<Syms>>;

pub type QasanFrontend =
    DefaultFrontend<BE, HostShadow<LibcHost<Syms>>, HostTracking<LibcHost<Syms>>>;

pub type QasanSyms = DlSymSymbols<LookupTypeNext>;

const PAGE_SIZE: usize = 4096;

static FRONTEND: Lazy<Mutex<QasanFrontend>> = Lazy::new(|| {
    LibcLogger::initialize::<QasanSyms>(Level::Info);
    let backend = BE::new(PAGE_SIZE);
    let shadow = HostShadow::<LibcHost<Syms>>::new().unwrap();
    let tracking = HostTracking::<LibcHost<Syms>>::new().unwrap();
    let frontend = QasanFrontend::new(
        backend,
        shadow,
        tracking,
        QasanFrontend::DEFAULT_REDZONE_SIZE,
        QasanFrontend::DEFAULT_QUARANTINE_SIZE,
    )
    .unwrap();
    Mutex::new(frontend)
});

#[no_mangle]
pub extern "C" fn asan_load(addr: *const c_void, size: usize) {
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
pub extern "C" fn asan_store(addr: *const c_void, size: usize) {
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
pub extern "C" fn asan_alloc(len: usize, align: usize) -> *mut c_void {
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
pub extern "C" fn asan_dealloc(addr: *const c_void) {
    trace!("free - addr: {:p}", addr);
    FRONTEND.lock().dealloc(addr as GuestAddr).unwrap();
}

#[no_mangle]
pub extern "C" fn asan_get_size(addr: *const c_void) -> usize {
    trace!("get_size - addr: {:p}", addr);
    FRONTEND.lock().get_size(addr as GuestAddr).unwrap()
}

#[no_mangle]
pub extern "C" fn asan_sym(name: *const c_char) -> GuestAddr {
    QasanSyms::lookup(name).unwrap()
}

#[no_mangle]
pub extern "C" fn asan_page_size() -> usize {
    PAGE_SIZE
}

#[no_mangle]
pub extern "C" fn asan_unpoison(addr: *const c_void, len: usize) {
    trace!("unpoison - addr: {:p}, len: {:#x}", addr, len);
    FRONTEND
        .lock()
        .shadow_mut()
        .unpoison(addr as GuestAddr, len)
        .unwrap();
}

#[no_mangle]
pub extern "C" fn asan_track(addr: *const c_void, len: usize) {
    trace!("track - addr: {:p}, len: {:#x}", addr, len);
    FRONTEND
        .lock()
        .tracking_mut()
        .alloc(addr as GuestAddr, len)
        .unwrap();
}

#[no_mangle]
pub extern "C" fn asan_untrack(addr: *const c_void) {
    trace!("untrack - addr: {:p}", addr);
    FRONTEND
        .lock()
        .tracking_mut()
        .dealloc(addr as GuestAddr)
        .unwrap();
}

#[no_mangle]
pub extern "C" fn asan_panic(msg: *const c_char) -> ! {
    trace!("panic - msg: {:p}", msg);
    let msg = unsafe { CStr::from_ptr(msg as *const c_char) };
    panic!("{}", msg.to_str().unwrap());
}
