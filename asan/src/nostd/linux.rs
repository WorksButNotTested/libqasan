use rustix::process::{kill_current_process_group, Signal};

pub fn die() -> ! {
    kill_current_process_group(Signal::Abort).unwrap();
    unreachable!();
}
