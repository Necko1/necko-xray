use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const PID_FILE: &str = "/tmp/necko-xray.pid";

pub fn acquire_lock() -> anyhow::Result<()> {
    if Path::new(PID_FILE).exists() {
        let mut file = File::open(PID_FILE)?;
        let mut pid_str = String::new();
        file.read_to_string(&mut pid_str)?;

        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            if is_process_running(pid) {
                anyhow::bail!("Daemon already running with PID: {}", pid);
            }
        }

        fs::remove_file(PID_FILE)?;
    }

    let pid = std::process::id();
    let mut file = File::create(PID_FILE)?;
    file.write_all(pid.to_string().as_bytes())?;

    Ok(())
}

pub fn release_lock() {
    let _ = fs::remove_file(PID_FILE);
}

pub fn get_daemon_pid() -> Option<i32> {
    let mut file = File::open(PID_FILE).ok()?;
    let mut pid_str = String::new();
    file.read_to_string(&mut pid_str).ok()?;
    pid_str.trim().parse().ok()
}

#[cfg(target_os = "linux")]
fn is_process_running(pid: i32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

#[cfg(not(target_os = "linux"))]
fn is_process_running(pid: i32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    kill(Pid::from_raw(pid), Signal::SIGNULL).is_ok()
}