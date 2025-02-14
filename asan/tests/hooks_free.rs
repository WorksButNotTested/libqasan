#[cfg(test)]
#[cfg(all(feature = "hooks"))]
mod tests {
    use {
        asan::hooks::{free::free, malloc::malloc},
        core::ptr::null_mut,
    };

    #[test]
    fn test_free_null() {
        unsafe { free(null_mut()) };
    }

    #[test]
    fn test_free_buff() {
        let p = unsafe { malloc(10) };
        unsafe { free(p) }
    }
}
