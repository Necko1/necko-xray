use std::process::Stdio;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("[necko-xray]: Starting Xray...");

    let mut child = Command::new("/usr/local/bin/xray")
        .arg("-config")
        .arg("/etc/xray/config.json")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to start Xray process");

    let pid = child.id().unwrap();
    println!("[necko-xray]: Xray started with PID: {}", pid);

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        status = child.wait() => {
            match status {
                Ok(s) => println!("[necko-xray]: Xray exited unexpectedly with {}", s),
                Err(e) => println!("[necko-xray]: Error waiting for Xray: {}", e),
            }
        }

        _ = sigterm.recv() => {
            println!("[necko-xray]: Received SIGTERM. Shutting down...");
        }

        _ = sigint.recv() => {
            println!("[necko-xray]: Received SIGINT (Ctrl+C). Shutting down...");
        }
    }

    Ok(())
}
