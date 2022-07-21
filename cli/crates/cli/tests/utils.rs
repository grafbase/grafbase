use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

use sysinfo::{Pid, ProcessExt, Signal, System, SystemExt};

/// # Panics
///
/// panics if the set timeout is reached
pub fn poll_endpoint(endpoint: &str, timeout_secs: u64, interval_millis: u64) {
    let client = reqwest::blocking::Client::new();

    let start = SystemTime::now();

    loop {
        if client.head(endpoint).send().is_ok() {
            break;
        }

        assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");

        sleep(Duration::from_millis(interval_millis));
    }
}

pub fn kill_with_children(pid: u32) {
    #[cfg(target_family = "unix")]
    let target_id = Pid::from(pid as i32);
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
