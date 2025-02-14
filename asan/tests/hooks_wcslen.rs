#[cfg(test)]
#[cfg(feature = "hooks")]
mod tests {
    use {
        asan::hooks::{expect_panic, wchar_t, wcslen::wcslen},
        core::ptr::null,
        widestring::widecstr,
    };

    #[test]
    fn test_wcslen_cs_null() {
        expect_panic();
        unsafe { wcslen(null() as *const wchar_t) };
        unreachable!();
    }

    #[test]
    fn test_wcslen_cs_empty() {
        let data = widecstr!("");
        let ret = unsafe { wcslen(data.as_ptr() as *const wchar_t) };
        assert_eq!(ret, 0);
    }

    #[test]
    fn test_wcslen_full() {
        let data = widecstr!("abcdefghij");
        let ret = unsafe { wcslen(data.as_ptr() as *const wchar_t) };
        assert_eq!(ret, 10);
    }
}
