use {
    alloc::format,
    log::{Level, LevelFilter, Log, Metadata, Record},
    rustix::{io::write, stdio::stdout},
    spin::Mutex,
};

static LOGGER: LinuxLogger = LinuxLogger::new();
pub struct LinuxLogger {
    level: Mutex<Level>,
}

impl LinuxLogger {
    const fn new() -> LinuxLogger {
        LinuxLogger {
            level: Mutex::new(Level::Info),
        }
    }

    pub fn initialize(level: Level) {
        *LOGGER.level.lock() = level;
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info)
    }
}

impl Log for LinuxLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        *self.level.lock() >= metadata.level()
    }

    fn flush(&self) {}

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let formatted = format!("{}\n", record.args());
            let buf = formatted.as_bytes();
            let fd = unsafe { stdout() };
            write(fd, buf).unwrap();
        }
    }
}
