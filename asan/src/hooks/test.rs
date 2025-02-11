use {
    crate::{
        allocator::{
            backend::dlmalloc::DlmallocBackend,
            frontend::{default::DefaultFrontend, AllocatorFrontend},
        },
        exit::exit,
        logger::linux::LinuxLogger,
        mmap::linux::LinuxMmap,
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
    core::{
        ffi::{c_char, c_void, CStr},
        sync::atomic::{AtomicBool, Ordering},
    },
    log::{error, trace, Level},
    spin::{Lazy, Mutex},
};

pub type TestFrontend = DefaultFrontend<
    DlmallocBackend<LinuxMmap>,
    GuestShadow<LinuxMmap, DefaultShadowLayout>,
    GuestTracking,
>;

pub type TestSyms = DlSymSymbols<LookupTypeNext>;

const PAGE_SIZE: usize = 4096;

static FRONTEND: Lazy<Mutex<TestFrontend>> = Lazy::new(|| {
    LinuxLogger::initialize(Level::Info);
    let backend = DlmallocBackend::<LinuxMmap>::new(PAGE_SIZE);
    let shadow = GuestShadow::<LinuxMmap, DefaultShadowLayout>::new().unwrap();
    let tracking = GuestTracking::new().unwrap();
    let frontend = TestFrontend::new(
        backend,
        shadow,
        tracking,
        TestFrontend::DEFAULT_REDZONE_SIZE,
        TestFrontend::DEFAULT_QUARANTINE_SIZE,
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
    TestSyms::lookup(name).unwrap()
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

static EXPECT_PANIC: AtomicBool = AtomicBool::new(false);

pub fn expect_panic() {
    EXPECT_PANIC.store(true, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn asan_panic(msg: *const c_char) -> ! {
    trace!("panic - msg: {:p}", msg);
    let msg = unsafe { CStr::from_ptr(msg as *const c_char) };
    error!("{}", msg.to_str().unwrap());
    match EXPECT_PANIC.load(Ordering::SeqCst) {
        true => {
            exit(0);
        }
        false => {
            panic!("unexpected panic");
        }
    }
}
