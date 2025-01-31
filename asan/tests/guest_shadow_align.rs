#[cfg(test)]
mod tests {
    use asan::shadow::guest::DefaultShadowLayout;
    #[cfg(feature = "guest")]
    use asan::{mmap::Mmap, shadow::guest::GuestShadow, GuestAddr};

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

    type GS = GuestShadow<DummyMmap, DefaultShadowLayout>;

    #[test]
    #[cfg(feature = "guest")]
    fn test_align_up_zero() {
        assert_eq!(GS::align_up(0), 0);
        assert_eq!(GS::align_up(1), 8);
        assert_eq!(GS::align_up(2), 8);
        assert_eq!(GS::align_up(3), 8);
        assert_eq!(GS::align_up(4), 8);
        assert_eq!(GS::align_up(5), 8);
        assert_eq!(GS::align_up(6), 8);
        assert_eq!(GS::align_up(7), 8);
    }

    #[test]
    #[cfg(feature = "guest")]
    #[should_panic]
    fn test_align_up_max_minus_one() {
        GS::align_up(GuestAddr::MAX - 1);
    }

    #[test]
    #[cfg(feature = "guest")]
    #[should_panic]
    fn test_align_up_max_minus_two() {
        GS::align_up(GuestAddr::MAX - 2);
    }

    #[test]
    #[cfg(feature = "guest")]
    #[should_panic]
    fn test_align_up_max_minus_three() {
        GS::align_up(GuestAddr::MAX - 3);
    }

    #[test]
    #[cfg(feature = "guest")]
    #[should_panic]
    fn test_align_up_max_minus_four() {
        GS::align_up(GuestAddr::MAX - 4);
    }

    #[test]
    #[cfg(feature = "guest")]
    #[should_panic]
    fn test_align_up_max_minus_five() {
        GS::align_up(GuestAddr::MAX - 5);
    }

    #[test]
    #[cfg(feature = "guest")]
    #[should_panic]
    fn test_align_up_max_minus_six() {
        GS::align_up(GuestAddr::MAX - 6);
    }

    #[test]
    #[cfg(feature = "guest")]
    fn test_align_down_zero() {
        assert_eq!(GS::align_down(0), 0);
        assert_eq!(GS::align_down(1), 0);
        assert_eq!(GS::align_down(2), 0);
        assert_eq!(GS::align_down(3), 0);
        assert_eq!(GS::align_down(4), 0);
        assert_eq!(GS::align_down(5), 0);
        assert_eq!(GS::align_down(6), 0);
        assert_eq!(GS::align_down(7), 0);
    }

    #[test]
    #[cfg(feature = "guest")]
    fn test_align_down_max() {
        assert_eq!(GS::align_down(GuestAddr::MAX), GuestAddr::MAX - 7);
        assert_eq!(GS::align_down(GuestAddr::MAX - 1), GuestAddr::MAX - 7);
        assert_eq!(GS::align_down(GuestAddr::MAX - 2), GuestAddr::MAX - 7);
        assert_eq!(GS::align_down(GuestAddr::MAX - 3), GuestAddr::MAX - 7);
        assert_eq!(GS::align_down(GuestAddr::MAX - 4), GuestAddr::MAX - 7);
        assert_eq!(GS::align_down(GuestAddr::MAX - 5), GuestAddr::MAX - 7);
        assert_eq!(GS::align_down(GuestAddr::MAX - 6), GuestAddr::MAX - 7);
    }
}
