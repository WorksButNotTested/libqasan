use {
    crate::{
        asan_swap,
        symbols::{Function, FunctionPointer, Symbols, SymbolsLookupStr},
        GuestAddr,
    },
    alloc::{boxed::Box, format},
    core::ffi::{c_int, c_void, CStr},
    libc::{size_t, ssize_t, STDERR_FILENO},
    log::{Level, LevelFilter, Log, Metadata, Record},
    spin::Once,
};

#[derive(Debug)]
struct FunctionWrite;

impl Function for FunctionWrite {
    type Func = unsafe extern "C" fn(c_int, *const c_void, size_t) -> ssize_t;
    const NAME: &'static CStr = c"write";
}

static ONCE: Once<&'static LibcLogger> = Once::new();
pub struct LibcLogger {
    level: Level,
    write: GuestAddr,
}

impl LibcLogger {
    pub fn initialize<S: Symbols>(level: Level) {
        ONCE.call_once(|| {
            let write = S::lookup_str(FunctionWrite::NAME).unwrap();
            let logger = Box::leak(Box::new(LibcLogger { level, write }));
            log::set_logger(logger).unwrap();
            log::set_max_level(LevelFilter::Trace);
            logger
        });
    }
}

impl Log for LibcLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.level >= metadata.level()
    }

    fn flush(&self) {}

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let formatted = format!(
                "{} [{}]: {}\n",
                record.metadata().level(),
                record.metadata().target(),
                record.args()
            );
            let buf = formatted.as_bytes();
            let fn_write = FunctionWrite::as_ptr(self.write).unwrap();
            unsafe { asan_swap(false) };
            unsafe {
                fn_write(
                    STDERR_FILENO,
                    buf.as_ptr() as *const c_void,
                    buf.len() as size_t,
                )
            };
            unsafe { asan_swap(true) };
        }
    }
}
