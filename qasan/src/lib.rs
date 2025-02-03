use {
    asan::{
        allocator::frontend::{default::DefaultFrontend, Allocator},
        host::libc::LibcHost,
        shadow::host::HostShadow,
        tracking::host::HostTracking,
        GuestAddr,
    },
    std::{
        ptr::null_mut,
        sync::{LazyLock, Mutex},
    },
};

#[cfg(not(target_arch = "powerpc"))]
type BE = asan::allocator::backend::mimalloc::MimallocBackend;

#[cfg(target_arch = "powerpc")]
type BE =
    asan::allocator::backend::dlmalloc::DlmallocBackend<asan::mmap::libc::LibcMmap<asan::symbols::dlsym::DlSymSymbols<asan::symbols::dlsym::LookupTypeNext>>>;

pub type ZasanAllocator =
    DefaultFrontend<BE, HostShadow<LibcHost>, HostTracking<LibcHost>>;

static ALLOCATOR: LazyLock<Mutex<ZasanAllocator>> = LazyLock::new(|| {
    Mutex::new({
        let backend = BE::new();
        let shadow = HostShadow::<LibcHost>::new().unwrap();
        let tracking = HostTracking::<LibcHost>::new().unwrap();
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
    if let Ok(mut allocator) = ALLOCATOR.lock() {
        let addr = allocator.alloc(size, align).unwrap();
        addr as *mut u8
    } else {
        null_mut()
    }
}

#[no_mangle]
pub extern "C" fn zasan_deallocate(addr: *mut u8) -> bool {
    if let Ok(mut allocator) = ALLOCATOR.lock() {
        allocator.dealloc(addr as GuestAddr).is_ok()
    } else {
        false
    }
}
