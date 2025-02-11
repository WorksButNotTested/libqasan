#[cfg(test)]
#[cfg(feature = "hooks")]
mod tests {
    use asan::hooks::{aligned_alloc::aligned_alloc, expect_panic};

    #[test]
    fn aligned_alloc_size_not_multiple() {
        expect_panic();
        unsafe { aligned_alloc(9, 8) };
        unreachable!();
    }

    #[test]
    fn aligned_alloc_power_of_two() {
        let addr = unsafe { aligned_alloc(8, 8) };
        assert_ne!(addr, 0 as *mut _);
        assert_eq!(addr as usize & 7, 0);
    }

    #[test]
    fn aligned_alloc_not_power_of_two() {
        expect_panic();
        unsafe { aligned_alloc(7, 24) };
        unreachable!();
    }
}
