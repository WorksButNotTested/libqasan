#[cfg(test)]
mod tests {
    use {
        asan::{
            symbols::{
                dlsym::{DlSymSymbols, LookupTypeDefault},
                Symbols,
            },
            GuestAddr,
        },
        core::ffi::{c_int, c_void},
        libc::{off_t, size_t},
    };

    type FnMmap =
        unsafe extern "C" fn(*mut c_void, size_t, c_int, c_int, c_int, off_t) -> *mut c_void;
    type FnMunmap = unsafe extern "C" fn(*mut c_void, size_t) -> c_int;

    type DLSYM = DlSymSymbols<LookupTypeDefault>;

    #[test]
    #[cfg(feature = "libc")]
    fn test_dlsym() {
        let mmap: FnMmap = DLSYM::lookup("mmap").unwrap();
        let mmap2: FnMmap = DLSYM::lookup("mmap").unwrap();
        assert_eq!(mmap, mmap2);
        let mapping = unsafe {
            mmap(
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
        let munmap: FnMunmap = DLSYM::lookup("munmap").unwrap();
        let munmap2: FnMunmap = DLSYM::lookup("munmap").unwrap();
        assert_eq!(munmap, munmap2);
        let ret = unsafe { munmap(mapping, 4096) };
        assert!(ret == 0);
    }
}
