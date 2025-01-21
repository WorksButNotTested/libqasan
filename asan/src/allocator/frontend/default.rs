//! # default
//! The default frontend is primarily designed for simplicity. Though called
//! the default, it may be subsequently replaced as the preferred frontend
//! should a more optimal design be implemented at a later date.
//!
//! This frontend stores all of it's metadata out-of-band, that is no meta-data
//! is stored adjacent to the user's buffers. The size of the red-zone applied
//! to each allocation is configurable. The frontend also supports the use of a
//! quarantine (whose size is configurable) to prevent user buffers from being
//! re-used for a period of time.
use {
    crate::{
        allocator::{backend::AllocatorBackend, frontend::Allocator},
        shadow::{PoisonType, Shadow},
        tracking::Tracking,
        GuestAddr,
    },
    alloc::{
        collections::{BTreeMap, VecDeque},
        fmt::Debug,
    },
    core::slice::from_raw_parts_mut,
    log::debug,
    thiserror::Error,
};

struct Allocation {
    addr: GuestAddr,
    len: usize,
    align: usize,
}

pub struct DefaultFrontend<B: AllocatorBackend, S: Shadow, T: Tracking> {
    backend: B,
    shadow: S,
    tracking: T,
    red_zone_size: usize,
    allocations: BTreeMap<GuestAddr, Allocation>,
    quarantine: VecDeque<Allocation>,
    quarantine_size: usize,
    quaratine_used: usize,
}

impl<B: AllocatorBackend, S: Shadow, T: Tracking> Allocator for DefaultFrontend<B, S, T> {
    type Error = DefaultFrontendError<B, S, T>;

    fn alloc(&mut self, len: usize, align: usize) -> Result<GuestAddr, Self::Error> {
        debug!("alloc - len: 0x{:x}, align: 0x{:x}", len, align);
        if align % size_of::<GuestAddr>() != 0 {
            Err(DefaultFrontendError::InvalidAlignment(align))?;
        }
        let size = len + align;
        let allocated_size = self.red_zone_size + Self::align_up(size);
        assert!(allocated_size % Self::ALLOC_ALIGN_SIZE == 0);
        let orig = self
            .backend
            .alloc(allocated_size, Self::ALLOC_ALIGN_SIZE)
            .map_err(|e| DefaultFrontendError::AllocatorError(e))?;

        let rz = orig + self.red_zone_size;
        let data = if align == 0 {
            rz
        } else {
            rz + align - (rz % align)
        };
        assert!(align == 0 || data % align == 0);
        assert!(data + len <= orig + allocated_size);

        self.allocations.insert(
            data,
            Allocation {
                addr: orig,
                len: allocated_size,
                align: Self::ALLOC_ALIGN_SIZE,
            },
        );

        self.tracking
            .alloc(data, len)
            .map_err(|e| DefaultFrontendError::TrackingError(e))?;
        self.shadow
            .poison(rz, self.red_zone_size, PoisonType::AsanHeapLeftRz)
            .map_err(|e| DefaultFrontendError::ShadowError(e))?;
        let poison_len = Self::align_up(len) - len + self.red_zone_size;
        self.shadow
            .poison(data + len, poison_len, PoisonType::AsanStackRightRz)
            .map_err(|e| DefaultFrontendError::ShadowError(e))?;

        let buffer = unsafe { from_raw_parts_mut(data as *mut u8, len) };
        buffer.iter_mut().for_each(|b| *b = 0xff);
        Ok(data)
    }

    fn dealloc(&mut self, addr: GuestAddr) -> Result<(), Self::Error> {
        debug!("dealloc - addr: 0x{:x}", addr);
        if addr == 0 {
            return Ok(());
        }

        let alloc = self
            .allocations
            .remove(&addr)
            .ok_or_else(|| DefaultFrontendError::InvalidAddress(addr))?;
        self.shadow
            .poison(alloc.addr, alloc.len, PoisonType::AsanHeapFreed)
            .map_err(|e| DefaultFrontendError::ShadowError(e))?;
        self.tracking
            .dealloc(addr)
            .map_err(|e| DefaultFrontendError::TrackingError(e))?;
        self.quaratine_used += alloc.len;
        self.quarantine.push_back(alloc);
        self.purge_quarantine()?;
        Ok(())
    }
}

impl<B: AllocatorBackend, S: Shadow, T: Tracking> DefaultFrontend<B, S, T> {
    #[cfg(target_pointer_width = "32")]
    const ALLOC_ALIGN_SIZE: usize = 8;

    #[cfg(target_pointer_width = "64")]
    const ALLOC_ALIGN_SIZE: usize = 16;

    pub const DEFAULT_REDZONE_SIZE: usize = 128;
    pub const DEFAULT_QUARANTINE_SIZE: usize = 50 << 20;

    pub fn new(
        backend: B,
        shadow: S,
        tracking: T,
        red_zone_size: usize,
        quarantine_size: usize,
    ) -> Result<DefaultFrontend<B, S, T>, DefaultFrontendError<B, S, T>> {
        if red_zone_size % Self::ALLOC_ALIGN_SIZE != 0 {
            Err(DefaultFrontendError::InvalidRedZoneSize(red_zone_size))?;
        }
        Ok(DefaultFrontend::<B, S, T> {
            backend,
            shadow,
            tracking,
            red_zone_size,
            allocations: BTreeMap::new(),
            quarantine: VecDeque::new(),
            quarantine_size,
            quaratine_used: 0,
        })
    }

    fn purge_quarantine(&mut self) -> Result<(), DefaultFrontendError<B, S, T>> {
        while self.quaratine_used > self.quarantine_size {
            let alloc = self
                .quarantine
                .pop_front()
                .ok_or(DefaultFrontendError::QuarantineCorruption)?;
            self.backend
                .dealloc(alloc.addr, alloc.len, alloc.align)
                .map_err(|e| DefaultFrontendError::AllocatorError(e))?;
        }
        Ok(())
    }

    fn align_up(size: usize) -> usize {
        assert!(size <= GuestAddr::MAX - (Self::ALLOC_ALIGN_SIZE - 1));
        let val = size + (Self::ALLOC_ALIGN_SIZE - 1);
        val & !(Self::ALLOC_ALIGN_SIZE - 1)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum DefaultFrontendError<B: AllocatorBackend, S: Shadow, T: Tracking> {
    #[error("Invalid red_zone_size: {0}")]
    InvalidRedZoneSize(usize),
    #[error("Invalid alignment: {0}")]
    InvalidAlignment(usize),
    #[error("Allocator error: {0:?}")]
    AllocatorError(B::Error),
    #[error("Shadow error: {0:?}")]
    ShadowError(S::Error),
    #[error("Tracking error: {0:?}")]
    TrackingError(T::Error),
    #[error("Invalid address: {0:x}")]
    InvalidAddress(GuestAddr),
    #[error("Quarantine corruption")]
    QuarantineCorruption,
}
