#[cfg(test)]
mod tests {
    #[cfg(feature = "guest")]
    use asan::{mmap::Mmap, shadow::guest::GuestShadow};

    #[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
    struct DummyMmap;

    impl Mmap for DummyMmap {
        type Error = DummyMmapError;

        fn map(_size: usize) -> Result<Self, Self::Error> {
            unimplemented!()
        }

        fn map_at(_base: asan::GuestAddr, _size: usize) -> Result<Self, Self::Error> {
            unimplemented!()
        }

        fn as_slice(&self) -> &[u8] {
            unimplemented!()
        }

        fn as_mut_slice(&mut self) -> &mut [u8] {
            unimplemented!()
        }
    }

    #[derive(Debug)]
    struct DummyMmapError;

    type GS = GuestShadow<DummyMmap>;

    #[test]
    #[cfg(feature = "guest")]
    fn test_low_mem() {
        assert!(GS::is_low_memory(GS::LOW_MEM_OFFSET, 1));
        assert!(GS::is_low_memory(GS::LOW_MEM_OFFSET, GS::LOW_MEM_SIZE));
        assert!(!GS::is_low_memory(GS::LOW_MEM_OFFSET, GS::LOW_MEM_SIZE + 1));
        assert!(GS::is_low_memory(GS::LOW_MEM_LIMIT, 1));

        assert!(!GS::is_high_memory(GS::LOW_MEM_OFFSET, 1));
        assert!(!GS::is_high_memory(GS::LOW_MEM_OFFSET, GS::LOW_MEM_SIZE));
        assert!(!GS::is_high_memory(
            GS::LOW_MEM_OFFSET,
            GS::LOW_MEM_SIZE + 1
        ));
        assert!(!GS::is_high_memory(GS::LOW_MEM_LIMIT, 1));

        assert!(GS::is_memory(GS::LOW_MEM_OFFSET, 1));
        assert!(GS::is_memory(GS::LOW_MEM_OFFSET, GS::LOW_MEM_SIZE));
        assert!(!GS::is_memory(GS::LOW_MEM_OFFSET, GS::LOW_MEM_SIZE + 1));
        assert!(GS::is_memory(GS::LOW_MEM_LIMIT, 1));
    }

    #[test]
    #[cfg(feature = "guest")]
    fn test_low_shadow() {
        assert!(!GS::is_low_memory(GS::LOW_SHADOW_OFFSET, 1));
        assert!(!GS::is_low_memory(
            GS::LOW_SHADOW_OFFSET,
            GS::LOW_SHADOW_SIZE
        ));
        assert!(!GS::is_low_memory(GS::LOW_SHADOW_LIMIT, 1));

        assert!(!GS::is_high_memory(GS::LOW_SHADOW_OFFSET, 0));
        assert!(!GS::is_high_memory(
            GS::LOW_SHADOW_OFFSET,
            GS::LOW_SHADOW_SIZE
        ));
        assert!(!GS::is_high_memory(GS::LOW_SHADOW_LIMIT, 1));

        assert!(!GS::is_memory(GS::LOW_SHADOW_OFFSET, 1));
        assert!(!GS::is_memory(GS::LOW_SHADOW_OFFSET, GS::LOW_SHADOW_SIZE));
        assert!(!GS::is_memory(GS::LOW_SHADOW_LIMIT, 1));
    }

    #[test]
    #[cfg(feature = "guest")]
    fn test_high_shadow() {
        assert!(!GS::is_low_memory(GS::HIGH_SHADOW_OFFSET, 1));
        assert!(!GS::is_low_memory(
            GS::HIGH_SHADOW_OFFSET,
            GS::HIGH_SHADOW_SIZE
        ));
        assert!(!GS::is_low_memory(GS::HIGH_SHADOW_LIMIT, 1));

        assert!(!GS::is_high_memory(GS::HIGH_SHADOW_OFFSET, 0));
        assert!(!GS::is_high_memory(
            GS::HIGH_SHADOW_OFFSET,
            GS::HIGH_SHADOW_SIZE
        ));
        assert!(!GS::is_high_memory(GS::HIGH_SHADOW_LIMIT, 1));

        assert!(!GS::is_memory(GS::HIGH_SHADOW_OFFSET, 1));
        assert!(!GS::is_memory(GS::HIGH_SHADOW_OFFSET, GS::HIGH_SHADOW_SIZE));
        assert!(!GS::is_memory(GS::HIGH_SHADOW_LIMIT, 1));
    }

    #[test]
    #[cfg(feature = "guest")]
    fn test_high_mem() {
        assert!(GS::is_high_memory(GS::HIGH_MEM_OFFSET, 1));
        assert!(GS::is_high_memory(GS::HIGH_MEM_OFFSET, GS::HIGH_MEM_SIZE));
        assert!(!GS::is_high_memory(
            GS::HIGH_MEM_OFFSET,
            GS::HIGH_MEM_SIZE + 1
        ));
        assert!(GS::is_high_memory(GS::HIGH_MEM_LIMIT, 1));

        assert!(!GS::is_low_memory(GS::HIGH_MEM_OFFSET, 1));
        assert!(!GS::is_low_memory(GS::HIGH_MEM_OFFSET, GS::HIGH_MEM_SIZE));
        assert!(!GS::is_low_memory(
            GS::HIGH_MEM_OFFSET,
            GS::HIGH_MEM_SIZE + 1
        ));
        assert!(!GS::is_low_memory(GS::HIGH_MEM_LIMIT, 1));

        assert!(GS::is_memory(GS::HIGH_MEM_OFFSET, 1));
        assert!(GS::is_memory(GS::HIGH_MEM_OFFSET, GS::HIGH_MEM_SIZE));
        assert!(!GS::is_memory(GS::HIGH_MEM_OFFSET, GS::HIGH_MEM_SIZE + 1));
        assert!(GS::is_memory(GS::HIGH_MEM_LIMIT, 1));
    }
}
