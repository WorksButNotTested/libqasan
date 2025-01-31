#[cfg(test)]
mod tests {
    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    use {
        asan::{
            mmap::libc::LibcMmap,
            shadow::{
                guest::{DefaultShadowLayout, GuestShadow},
                PoisonType, Shadow,
            },
        },
        spin::Lazy,
        std::sync::Mutex,
    };

    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    type GS = GuestShadow<LibcMmap, DefaultShadowLayout>;

    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    static INIT_ONCE: Lazy<Mutex<()>> = Lazy::new(|| {
        {
            env_logger::init();
        };
        Mutex::new(())
    });

    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    fn get_shadow() -> GuestShadow<LibcMmap, DefaultShadowLayout> {
        drop(INIT_ONCE.lock().unwrap());
        GS::new().unwrap()
    }

    #[test]
    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    fn test_poison_example1() {
        let mut shadow = get_shadow();
        // poison - start: 0x7fff2bffff00, len: 0x100, pioson: AsanUser
        // is_poison - start: 0x7fff2bfffc01, len: 0x300
        assert_eq!(
            shadow.poison(0x7fff2bffff00, 0x100, PoisonType::AsanUser),
            Ok(())
        );
        assert_eq!(shadow.is_poison(0x7fff2bfffc01, 0x300), Ok(true));
    }

    #[test]
    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    fn test_poison_example2() {
        let mut shadow = get_shadow();
        // poison - start: 0x7dff13ffffff, len: 0x3b9, pioson: AsanUser
        // is_poison - start: 0x7dff14000302, len: 0x2
        assert_eq!(
            shadow.poison(0x7dff13ffffff, 0x3b9, PoisonType::AsanUser),
            Ok(())
        );
        assert_eq!(shadow.is_poison(0x7dff14000302, 0x2), Ok(true));
    }

    #[test]
    #[cfg(feature = "guest")]
    #[cfg(target_pointer_width = "64")]
    fn test_poison_example3() {
        let mut shadow = get_shadow();
        // poison - start: 0x7fffffffff00, len: 0x100, pioson: AsanUser
        // is_poison - start: 0x7fffffffff00, len: 0xff
        assert_eq!(
            shadow.poison(0x7fffffffff00, 0x100, PoisonType::AsanUser),
            Ok(())
        );
        assert_eq!(shadow.is_poison(0x7fffffffff00, 0xff), Ok(true));
    }
}
