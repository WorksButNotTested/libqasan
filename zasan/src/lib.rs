#![no_std]
extern crate alloc;

use {
    asan::{
        allocator::{
            backend::dlmalloc::DlmallocBackend,
            frontend::{default::DefaultFrontend, AllocatorFrontend},
        },
        asan_alloc, asan_dealloc, asan_init,
        logger::linux::LinuxLogger,
        mmap::linux::LinuxMmap,
        shadow::{
            guest::{DefaultShadowLayout, GuestShadow},
            Shadow,
        },
        symbols::{nop::NopSymbols, Symbol, Symbols},
        tracking::guest::GuestTracking,
        Asan, GuestAddr,
    },
    core::ffi::c_void,
    ctor::ctor,
    log::{trace, Level},
};

pub type ZasanAllocator = DefaultFrontend<
    DlmallocBackend<LinuxMmap>,
    GuestShadow<LinuxMmap, DefaultShadowLayout>,
    GuestTracking,
>;

pub type ZasanSyms = NopSymbols;

const PAGE_SIZE: usize = 4096;

struct Zasan {
    allocator: ZasanAllocator,
}

impl Asan for Zasan {
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
        ZasanSyms::lookup(name).unwrap()
    }

    fn asan_page_size(&self) -> usize {
        PAGE_SIZE
    }
}

#[ctor]
#[no_mangle]
fn zasan_init() {
    init();
}

pub extern "C" fn init() {
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
    let zasan = Zasan { allocator };
    asan_init(zasan);
}

#[no_mangle]
pub extern "C" fn zasan_allocate(size: usize, align: usize) -> *mut c_void {
    asan_alloc(size, align)
}

#[no_mangle]
pub extern "C" fn zasan_deallocate(addr: *mut c_void) {
    asan_dealloc(addr);
}
