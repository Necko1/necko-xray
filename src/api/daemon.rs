use std::ops::Deref;
use std::process::Stdio;
use tokio::net::UnixListener;
use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal::unix::{signal, SignalKind};

pub const SOCKET_PATH: &str = "/tmp/necko-xray.sock";

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

    let _ = std::fs::remove_file(SOCKET_PATH);
    Ok(())
}

async fn run_api_server() -> anyhow::Result<()> {
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("[necko-xray]: API Server listening on {}", SOCKET_PATH);

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            if let Ok(n) = stream.read(&mut buf).await {
                let cmd = String::from_utf8_lossy(&buf[..n]);
                println!("[necko-xray]: Received command: {}", cmd);

                let response = crate::api::handle_command(cmd.deref()).await
                    .unwrap_or_else(|e| e.to_string());

                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
    }
}