use {log::error, crate::nostd::die};

#[no_mangle]
extern "C" fn __aeabi_unwind_cpp_pr0() {
    error!("__aeabi_unwind_cpp_pr0");
    die();
}

#[no_mangle]
extern "C" fn __aeabi_unwind_cpp_pr1() {
    error!("__aeabi_unwind_cpp_pr1");
    die();
}
