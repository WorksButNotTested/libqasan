#[cfg(test)]
#[cfg(feature = "libc")]
mod tests {
    use {
        asan::{
            symbols::{
                dlsym::{DlSymSymbols, LookupTypeDefault},
                Function, SymbolsLookupStr,
            },
            GuestAddr,
        },
        core::ffi::{c_int, c_void},
        libc::{off_t, size_t},
    };

    #[derive(Debug)]
    struct FunctionMmap;

    impl Function for FunctionMmap {
        type Func =
            unsafe extern "C" fn(*mut c_void, size_t, c_int, c_int, c_int, off_t) -> *mut c_void;
        const NAME: &'static str = "mmap\0";
    }

    #[derive(Debug)]
    struct FunctionMunmap;

    impl Function for FunctionMunmap {
        type Func = unsafe extern "C" fn(*mut c_void, size_t) -> c_int;
        const NAME: &'static str = "munmap\0";
    }

    type DLSYM = DlSymSymbols<LookupTypeDefault>;

    #[test]
    fn test_dlsym() {
        use asan::symbols::FunctionPointer;

        let mmap = DLSYM::lookup_str("mmap\0").unwrap();
        let mmap2 = DLSYM::lookup_str("mmap\0").unwrap();
        assert_eq!(mmap, mmap2);
        let fnmmap = FunctionMmap::as_ptr(mmap).unwrap();
        let mapping = unsafe {
            fnmmap(
                core::ptr::null_mut(),
                4096,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };
        let addr = mapping as GuestAddr;
        assert!(addr & 0xfff == 0);
        let munmap = DLSYM::lookup_str("munmap\0").unwrap();
        let munmap2 = DLSYM::lookup_str("munmap\0").unwrap();
        assert_eq!(munmap, munmap2);
        let fnmunmap = FunctionMunmap::as_ptr(munmap).unwrap();
        let ret = unsafe { fnmunmap(mapping, 4096) };
        assert!(ret == 0);
    }
}
