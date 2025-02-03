use crate::shadow::guest::ShadowLayout;
use crate::GuestAddr;

mod host_info {
    include!(concat!(env!("OUT_DIR"), "/host_info.rs"));
}

pub use host_info::*;

#[derive(Debug)]
pub struct GuessedShadowLayout;

impl ShadowLayout for GuessedShadowLayout {
    // [x                   , b] 	HighMem
    // [a = 0x02008fff7000  , x] 	HighShadow
    // [0x00008fff7000, 0x02008fff6fff] 	ShadowGap
    // [0x00007fff8000, 0x00008fff6fff] 	LowShadow
    // [0x000000000000, 0x00007fff7fff] 	LowMem
    const SHADOW_OFFSET: usize = 0x7fff8000;

    const LOW_MEM_OFFSET: GuestAddr = 0x0;
    const LOW_MEM_SIZE: usize = 0x00007fff8000;

    const LOW_SHADOW_OFFSET: GuestAddr = 0x00007fff8000;
    const LOW_SHADOW_SIZE: usize = 0xffff000;

    const HIGH_SHADOW_OFFSET: GuestAddr = 0x02008fff7000;
    const HIGH_SHADOW_SIZE: usize = host_info::HOST_INFO_HIGH_SHADOW_SIZE;

    const HIGH_MEM_OFFSET: GuestAddr = host_info::HOST_INFO_HIGH_MEM_OFFSET;
    const HIGH_MEM_SIZE: usize = host_info::HOST_INFO_HIGH_MEM_SIZE;

    const ALLOC_ALIGN_POW: usize = 3;
    const ALLOC_ALIGN_SIZE: usize = 1 << Self::ALLOC_ALIGN_POW;
}
