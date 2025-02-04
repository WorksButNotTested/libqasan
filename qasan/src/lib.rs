#![no_std]

use {
    asan::{
        allocator::frontend::{default::DefaultFrontend, AllocatorFrontend},
        asan_alloc, asan_dealloc, asan_init,
        host::libc::LibcHost,
        logger::linux::LinuxLogger,
        shadow::{host::HostShadow, Shadow},
        symbols::{
            cached::CachedSymbols,
            dlsym::{DlSymSymbols, LookupTypeNext},
            Symbol, Symbols,
        },
        tracking::host::HostTracking,
        Asan, GuestAddr,
    },
    core::ffi::c_void,
    ctor::ctor,
    log::{trace, Level},
};

type Syms = CachedSymbols<DlSymSymbols<LookupTypeNext>>;

type BE = asan::allocator::backend::dlmalloc::DlmallocBackend<asan::mmap::libc::LibcMmap<Syms>>;

pub type QasanAllocator =
    DefaultFrontend<BE, HostShadow<LibcHost<Syms>>, HostTracking<LibcHost<Syms>>>;

pub type QasanSyms = CachedSymbols<DlSymSymbols<LookupTypeNext>>;

struct Qasan {
    allocator: QasanAllocator,
}

const PAGE_SIZE: usize = 4096;

impl Asan for Qasan {
    fn asan_load(&mut self, addr: *const c_void, size: usize) {
        trace!("load - addr: 0x{:x}, size: {:#x}", addr as GuestAddr, size);
        if self
            .allocator
            .shadow()
            .is_poison(addr as GuestAddr, size)
            .unwrap()
        {
            panic!("Poisoned - addr: 0x{:p}, size: 0x{:x}", addr, size);
        }
    }

    fn asan_store(&mut self, addr: *const c_void, size: usize) {
        trace!("store - addr: 0x{:x}, size: {:#x}", addr as GuestAddr, size);
        if self
            .allocator
            .shadow()
            .is_poison(addr as GuestAddr, size)
            .unwrap()
        {
            panic!("Poisoned - addr: 0x{:p}, size: 0x{:x}", addr, size);
        }
    }

    fn asan_alloc(&mut self, len: usize, align: usize) -> *mut c_void {
        trace!("alloc - len: {:#x}, align: {:#x}", len, align);
        self.allocator.alloc(len, align).unwrap() as *mut c_void
    }

    fn asan_dealloc(&mut self, addr: *const c_void) {
        trace!("free - addr: 0x{:p}", addr);
        self.allocator.dealloc(addr as GuestAddr).unwrap();
    }

    fn asan_get_size(&mut self, addr: *const c_void) -> usize {
        trace!("get_size - addr: 0x{:p}", addr);
        self.allocator.get_size(addr as GuestAddr).unwrap()
    }

    fn asan_sym(&mut self, name: &'static str) -> Symbol {
        QasanSyms::lookup(name).unwrap()
    }

    fn asan_page_size(&self) -> usize {
        PAGE_SIZE
    }
}

#[ctor]
#[no_mangle]
fn qasan_init() {
    init();
}

pub extern "C" fn init() {
    LinuxLogger::initialize(Level::Info);
    let backend = BE::new(PAGE_SIZE);
    let shadow = HostShadow::<LibcHost<Syms>>::new().unwrap();
    let tracking = HostTracking::<LibcHost<Syms>>::new().unwrap();
    let allocator = QasanAllocator::new(
        backend,
        shadow,
        tracking,
        QasanAllocator::DEFAULT_REDZONE_SIZE,
        QasanAllocator::DEFAULT_QUARANTINE_SIZE,
    )
    .unwrap();
    let qasan = Qasan { allocator };
    asan_init(qasan);
}

#[no_mangle]
pub extern "C" fn qasan_allocate(size: usize, align: usize) -> *mut c_void {
    asan_alloc(size, align)
}

#[no_mangle]
pub extern "C" fn qasan_deallocate(addr: *mut c_void) {
    asan_dealloc(addr);
}
