#![no_main]

use {
    asan::{
        mmap::libc::LibcMmap,
        shadow::{
            guest::{GuestShadow, GuestShadowError},
            PoisonType, Shadow,
        },
        GuestAddr,
    },
    lazy_static::lazy_static,
    libfuzzer_sys::fuzz_target,
    log::info,
    std::sync::{Mutex, MutexGuard},
};
type GS = GuestShadow<LibcMmap>;

lazy_static! {
    static ref INIT_ONCE: Mutex<GS> = Mutex::new({
        env_logger::init();
        GuestShadow::<LibcMmap>::new().unwrap()
    });
}

fn get_shadow() -> MutexGuard<'static, GS> {
    INIT_ONCE.lock().unwrap()
}

const MAX_LENGTH: usize = 0x3ff;
const MAX_ADDR: GuestAddr = GS::HIGH_MEM_LIMIT;
const MAX_OFFSET: usize = 0x7ff;

fuzz_target!(|data: Vec<GuestAddr>| {
    let mut shadow = get_shadow();
    if data.len() < 4 {
        return;
    }
    info!("data: {:x?}", data);
    let start = data[0] & MAX_ADDR;
    let len = data[1] & MAX_LENGTH;
    let test_offset = data[2] & MAX_OFFSET;
    let test_len = data[3] & MAX_LENGTH;

    let result = shadow.poison(start, len, PoisonType::AsanUser);

    if !GS::is_memory(start, len) {
        assert_eq!(result, Err(GuestShadowError::InvalidMemoryAddress(start)));
        return;
    } else if !GS::is_end_aligned(start, len) {
        assert_eq!(
            result,
            Err(GuestShadowError::UnalignedEndAddress(start, len,))
        );
        return;
    } else {
        assert_eq!(result, Ok(()));
    }

    let test_start = if test_offset > MAX_LENGTH {
        start.saturating_add(test_offset - MAX_LENGTH) & MAX_ADDR
    } else {
        start.saturating_sub(test_offset)
    };

    let test_result = shadow.is_poison(test_start, test_len);
    if !GS::is_memory(test_start, test_len) {
        assert_eq!(
            test_result,
            Err(GuestShadowError::InvalidMemoryAddress(test_start))
        );
    } else if len == 0 || test_len == 0 {
        assert_eq!(test_result, Ok(false));
    } else {
        let end = start + len;
        let test_end = test_start + test_len;
        let overlaps = !(test_end <= start || test_start >= end);
        assert_eq!(test_result, Ok(overlaps));
    }

    let start_aligned = GS::align_down(start);
    shadow
        .unpoison(start_aligned, len + start - start_aligned)
        .unwrap();

    let retest_result = shadow.is_poison(test_start, test_len);
    if !GS::is_memory(test_start, test_len) {
        assert_eq!(
            retest_result,
            Err(GuestShadowError::InvalidMemoryAddress(test_start))
        );
    } else {
        assert_eq!(retest_result, Ok(false));
    }
});
