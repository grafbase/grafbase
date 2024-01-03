use sysinfo::{Pid, Signal, System};

pub fn kill_with_children(pid: u32) {
    let target_id = Pid::from(pid as usize);

    let mut sys = System::new();
    sys.refresh_processes();

    let signal_preference = [Signal::Interrupt, Signal::Kill];
    let signal = signal_preference
        .iter()
        .find(|signal| sysinfo::SUPPORTED_SIGNALS.contains(signal))
        .unwrap();

    for (_, process) in sys
        .processes()
        .iter()
        .filter(|(pid, process)| process.parent() == Some(target_id) || **pid == target_id)
    {
        process.kill_with(*signal);
    }
}
