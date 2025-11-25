pub(crate) mod lock;

use crate::api::Request;
use bincode::config::standard;
use bincode::serde::decode_from_slice;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};

const PROFILE: &str = "example.json"; // todo change to sql query
pub const SOCKET_PATH: &str = "/tmp/necko-xray.sock";

pub async fn start() -> anyhow::Result<()> {
    println!("[necko-xray]: Starting daemon...");

    let profile_path = format!("/etc/xray/profiles/{}", PROFILE);
    let _ = if std::path::Path::new(&profile_path).exists() {
        crate::config::generate_config_from_profile(Some(&profile_path))
    } else {
        eprintln!("[necko-xray]: Cannot find {} profile ({})! Using empty profile",
                  PROFILE, profile_path);
        crate::config::generate_config_from_profile(None)
    };

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

    tokio::spawn(async move {
        if let Err(e) = run_api_server().await {
            eprintln!("[necko-xray]: API Server error: {}", e);
        }
    });

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = xray.wait() => println!("[necko-xray]: Xray exited"),
        _ = sigterm.recv() => println!("[necko-xray]: Received SIGTERM"),
        _ = sigint.recv() => println!("[necko-xray]: Received SIGINT"),
    }

    cleanup();
    Ok(())
}

fn cleanup() {
    let _ = std::fs::remove_file(SOCKET_PATH);
    lock::release_lock();
}

pub async fn stop() -> anyhow::Result<()> {
    if let Some(pid) = lock::get_daemon_pid() {
        println!("[necko-xray]: Stopping daemon (PID: {})...", pid);

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            kill(Pid::from_raw(pid), Signal::SIGTERM)?;
            println!("[necko-xray]: Stop signal sent successfully");
        }

        Ok(())
    } else {
        eprintln!("[necko-xray]: Daemon is not running");
        std::process::exit(1);
    }
}

async fn run_api_server() -> anyhow::Result<()> {
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("[necko-xray]: API Server listening on {}", SOCKET_PATH);

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_err() {
                return;
            }
            let len = u32::from_be_bytes(len_buf) as usize;

            let mut buf = vec![0u8; len];
            if stream.read_exact(&mut buf).await.is_err() {
                return;
            }

            let result: anyhow::Result<String> = {
                let (req, _read): (Request, usize) =
                    decode_from_slice(&buf, standard()).unwrap();
                crate::api::handle_command(req).await
            };

            let response = result.unwrap_or_else(|e| e.to_string());
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}