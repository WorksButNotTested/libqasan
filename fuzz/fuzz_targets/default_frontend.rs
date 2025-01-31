#![no_main]

use {
    asan::{
        allocator::{
            backend::dlmalloc::DlmallocBackend,
            frontend::{default::DefaultFrontend, Allocator},
        },
        mmap::linux::LinuxMmap,
        shadow::{
            guest::{DefaultShadowLayout, GuestShadow},
            Shadow,
        },
        tracking::guest::GuestTracking,
        GuestAddr,
    },
    libfuzzer_sys::fuzz_target,
    log::info,
    std::sync::{LazyLock, Mutex, MutexGuard},
};

type DF = DefaultFrontend<
    DlmallocBackend<LinuxMmap>,
    GuestShadow<LinuxMmap, DefaultShadowLayout>,
    GuestTracking,
>;

static INIT_ONCE: LazyLock<Mutex<DF>> = LazyLock::new(|| {
    env_logger::init();
    let backend = DlmallocBackend::<LinuxMmap>::new();
    let shadow = GuestShadow::<LinuxMmap, DefaultShadowLayout>::new().unwrap();
    let tracking = GuestTracking::new().unwrap();
    let frontend = DF::new(
        backend,
        shadow,
        tracking,
        DF::DEFAULT_REDZONE_SIZE,
        DF::DEFAULT_QUARANTINE_SIZE,
    )
    .unwrap();
    Mutex::new(frontend)
});

fn get_frontend() -> MutexGuard<'static, DF> {
    INIT_ONCE.lock().unwrap()
}

const MAX_LENGTH: usize = 0x3ff;
/*
 * Increase the changes of requesting unaligned or minimally aliugned allocations
 * since these are likely to be most common
 */
const ALIGNMENTS: [usize; 16] = [0, 0, 0, 0, 0, 8, 8, 8, 8, 16, 32, 64, 128, 256, 512, 1024];
const ALIGNMENTS_MASK: usize = ALIGNMENTS.len() - 1;

fuzz_target!(|data: Vec<GuestAddr>| {
    if data.len() < 2 {
        return;
    }
    let mut frontend = get_frontend();

    let len = data[0] & MAX_LENGTH;
    let align_idx = data[1] & ALIGNMENTS_MASK;
    let align = ALIGNMENTS[align_idx];

    info!("data: {:x?}, len: 0x{:x}, align: 0x{:x}", &data[0..2], len, align);

    if len == 0 {
        return;
    }

    let buf = frontend.alloc(len, align).unwrap();
    for i in buf - DF::DEFAULT_REDZONE_SIZE..buf {
        assert!(frontend.shadow.is_poison(i, 1).unwrap());
    }
    for i in buf..buf + len {
        assert!(!frontend.shadow.is_poison(i, 1).unwrap());
    }
    for i in buf + len..buf + len + DF::DEFAULT_REDZONE_SIZE {
        assert!(frontend.shadow.is_poison(i, 1).unwrap());
    }
    frontend.dealloc(buf).unwrap();
});
