#[cfg(test)]
mod tests {

    use {
        asan::{
            allocator::backend::{dlmalloc::DlmallocBackend, AllocatorBackend},
            mmap::linux::LinuxMmap,
        },
        spin::Lazy,
        std::sync::Mutex,
    };

    static INIT_ONCE: Lazy<Mutex<()>> = Lazy::new(|| {
        {
            env_logger::init();
        };
        Mutex::new(())
    });

    const PAGE_SIZE: usize = 4096;

    fn allocator() -> DlmallocBackend<LinuxMmap> {
        drop(INIT_ONCE.lock().unwrap());
        DlmallocBackend::<LinuxMmap>::new(PAGE_SIZE)
    }

    #[test]
    #[cfg(all(feature = "linux", feature = "dlmalloc"))]
    fn test_allocate() {
        let mut allocator = allocator();
        let buf = allocator.alloc(16, 8).unwrap();
        allocator.dealloc(buf, 16, 8).unwrap();
    }
}
