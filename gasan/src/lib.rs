use {
    asan::{
        allocator::{
            backend::mimalloc::MimallocBackend,
            frontend::{default::DefaultFrontend, Allocator},
        },
        mmap::libc::LibcMmap,
        shadow::guest::{DefaultShadowLayout, GuestShadow},
        symbols::dlsym::{DlSymSymbols, LookupTypeNext},
        tracking::guest::GuestTracking,
        GuestAddr,
    },
    std::{
        ptr::null_mut,
        sync::{LazyLock, Mutex},
    },
};

pub type ZasanAllocator = DefaultFrontend<
    MimallocBackend,
    GuestShadow<LibcMmap<DlSymSymbols<LookupTypeNext>>, DefaultShadowLayout>,
    GuestTracking,
>;

static ALLOCATOR: LazyLock<Mutex<ZasanAllocator>> = LazyLock::new(|| {
    Mutex::new({
        let backend = MimallocBackend::new();
        let shadow =
            GuestShadow::<LibcMmap<DlSymSymbols<LookupTypeNext>>, DefaultShadowLayout>::new()
                .unwrap();
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
