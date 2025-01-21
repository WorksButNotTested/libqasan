#[cfg(test)]
mod tests {
    #[cfg(feature = "guest")]
    use asan::host::linux::LinuxHost;
    #[test]
    #[cfg(feature = "guest")]
    fn test_sysno() {
        assert_eq!(LinuxHost::sysno() as u32, 0xa2a4);
    }
}
