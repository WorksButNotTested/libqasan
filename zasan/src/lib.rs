#![no_std]
mod arch;
mod mem;

use {
    asan::{
        allocator::{
            backend::dlmalloc::DlmallocBackend,
            frontend::{default::DefaultFrontend, Allocator},
        },
        mmap::linux::LinuxMmap,
        shadow::guest::{DefaultShadowLayout, GuestShadow},
        tracking::guest::GuestTracking,
        GuestAddr,
    },
    spin::{Lazy, Mutex},
};

pub type ZasanAllocator = DefaultFrontend<
    DlmallocBackend<LinuxMmap>,
    GuestShadow<LinuxMmap, DefaultShadowLayout>,
    GuestTracking,
>;

static ALLOCATOR: Lazy<Mutex<ZasanAllocator>> = Lazy::new(|| {
    Mutex::new({
        let backend = DlmallocBackend::<LinuxMmap>::new();
        let shadow = GuestShadow::<LinuxMmap, DefaultShadowLayout>::new().unwrap();
        let tracking = GuestTracking::new().unwrap();
        ZasanAllocator::new(
            backend,
            shadow,
            tracking,
            ZasanAllocator::DEFAULT_REDZONE_SIZE,
            ZasanAllocator::DEFAULT_QUARANTINE_SIZE,
        )
        .unwrap()
    })
});

#[no_mangle]
pub extern "C" fn zasan_allocate(size: usize, align: usize) -> *mut u8 {
    let mut allocator = ALLOCATOR.lock();
    let addr = allocator.alloc(size, align).unwrap();
    addr as *mut u8
}

#[no_mangle]
pub extern "C" fn zasan_deallocate(addr: *mut u8) -> bool {
    let mut allocator = ALLOCATOR.lock();
    allocator.dealloc(addr as GuestAddr).is_ok()
}
