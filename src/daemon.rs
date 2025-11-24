use std::process::Stdio;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};

const SOCKET_PATH: &str = "/tmp/necko-xray.sock";

pub async fn start() -> anyhow::Result<()> {
    println!("[necko-xray]: Starting daemon...");

    let mut xray = Command::new("/usr/local/bin/xray")
        .arg("-config")
        .arg("/etc/xray/config.json")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to start Xray process");

    let pid = xray.id().unwrap();
    println!("[necko-xray]: Xray started with PID: {}", pid);

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = xray.wait() => println!("[necko-xray]: Xray exited"),
        _ = sigterm.recv() => println!("[necko-xray]: Received SIGTERM"),
        _ = sigint.recv() => println!("[necko-xray]: Received SIGINT"),
    }

    let _ = std::fs::remove_file(SOCKET_PATH);
    Ok(())
}