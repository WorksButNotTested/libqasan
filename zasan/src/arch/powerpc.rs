use log::error;

#[no_mangle]
extern "C" fn _Unwind_Resume() {
    error!("_Unwind_Resume");
    loop {}
}
