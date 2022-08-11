use sysinfo::{Pid, ProcessExt, Signal, System, SystemExt};

pub fn kill_with_children(pid: u32) {
    #[cfg(target_family = "unix")]
    let target_id = Pid::from(i32::try_from(pid).unwrap());
    #[cfg(target_family = "windows")]
    let target_id = Pid::from(pid as usize);

    let mut sys = System::new();
    sys.refresh_processes();

    let signal_preference = vec![Signal::Interrupt, Signal::Kill];
    let signal = signal_preference
        .iter()
        .find(|signal| System::SUPPORTED_SIGNALS.contains(signal))
        .unwrap();

    for (_, process) in sys
        .processes()
        .iter()
        .filter(|(pid, process)| process.parent() == Some(target_id) || **pid == target_id)
    {
        process.kill_with(*signal);
    }
}
