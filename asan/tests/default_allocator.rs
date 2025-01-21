#[cfg(test)]
mod tests {

    use {
        asan::{
            allocator::{
                backend::dlmalloc::DlmallocBackend,
                frontend::{default::DefaultFrontend, Allocator},
            },
            mmap::linux::LinuxMmap,
            shadow::guest::GuestShadow,
            tracking::guest::GuestTracking,
        },
        spin::Lazy,
        std::sync::Mutex,
    };

    static INIT_ONCE: Lazy<Mutex<()>> = Lazy::new(|| {
        Mutex::new({
            env_logger::init();
            ()
        })
    });

    type DA = DefaultFrontend<DlmallocBackend<LinuxMmap>, GuestShadow<LinuxMmap>, GuestTracking>;

    fn allocator() -> DA {
        drop(INIT_ONCE.lock().unwrap());
        let backend = DlmallocBackend::<LinuxMmap>::new();
        let shadow = GuestShadow::<LinuxMmap>::new().unwrap();
        let tracking = GuestTracking::new().unwrap();
        DA::new(
            backend,
            shadow,
            tracking,
            DA::DEFAULT_REDZONE_SIZE,
            DA::DEFAULT_QUARANTINE_SIZE,
        )
        .unwrap()
    }

    #[test]
    #[cfg(all(feature = "linux", feature = "dlmalloc"))]
    fn test_allocate() {
        let mut allocator = allocator();
        let buf = allocator.alloc(16, 8).unwrap();
        allocator.dealloc(buf).unwrap();
    }
}
