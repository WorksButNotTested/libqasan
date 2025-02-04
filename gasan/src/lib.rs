#![no_std]
use {
    asan::{
        allocator::frontend::{default::DefaultFrontend, AllocatorFrontend},
        asan_alloc, asan_dealloc, asan_init,
        logger::linux::LinuxLogger,
        mmap::libc::LibcMmap,
        shadow::{
            guest::{DefaultShadowLayout, GuestShadow},
            Shadow,
        },
        symbols::{
            cached::CachedSymbols,
            dlsym::{DlSymSymbols, LookupTypeNext},
            Symbol, Symbols,
        },
        tracking::guest::GuestTracking,
        Asan, GuestAddr,
    },
    core::ffi::c_void,
    ctor::ctor,
    log::{trace, Level},
};

type Syms = CachedSymbols<DlSymSymbols<LookupTypeNext>>;

type BE = asan::allocator::backend::dlmalloc::DlmallocBackend<LibcMmap<Syms>>;

pub type GasanAllocator =
    DefaultFrontend<BE, GuestShadow<LibcMmap<Syms>, DefaultShadowLayout>, GuestTracking>;

pub type GasanSyms = CachedSymbols<DlSymSymbols<LookupTypeNext>>;

struct Gasan {
    allocator: GasanAllocator,
}

const PAGE_SIZE: usize = 4096;

impl Asan for Gasan {
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
        GasanSyms::lookup(name).unwrap()
    }

    fn asan_page_size(&self) -> usize {
        PAGE_SIZE
    }
}

#[ctor]
#[no_mangle]
fn gasan_init() {
    init();
}

pub extern "C" fn init() {
    LinuxLogger::initialize(Level::Info);
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
    let gasan = Gasan { allocator };
    asan_init(gasan);
}

#[no_mangle]
pub extern "C" fn gasan_allocate(size: usize, align: usize) -> *mut c_void {
    asan_alloc(size, align)
}

#[no_mangle]
pub extern "C" fn gasan_deallocate(addr: *mut c_void) {
    asan_dealloc(addr);
}
