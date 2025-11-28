pub mod lock;

use std::env;
use crate::api::Request;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use std::process::Stdio;
use sqlx::PgPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};

const PROFILE: &str = "example.json"; // todo change to sql query
pub const SOCKET_PATH: &str = "/tmp/necko-xray.sock";
const XRAY_PID_FILE: &str = "/tmp/necko-xray-core.pid";

pub fn is_xray_running() -> bool {
    if let Ok(pid_str) = std::fs::read_to_string(XRAY_PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            return lock::is_process_running(pid);
        }
    }
    false
}

pub async fn start() -> anyhow::Result<()> {
    println!("[necko-xray]: Starting daemon...");

    let profile_path = format!("/etc/xray/profiles/{}", PROFILE);
    let _ = if std::path::Path::new(&profile_path).exists() {
        crate::config::generate_config_from_profile(Some(&profile_path))
    } else {
        eprintln!(
            "[necko-xray]: Cannot find {} profile ({})! Using empty profile",
            PROFILE, profile_path
        );
        crate::config::generate_config_from_profile(None)
    };

    // start xray
    start_xray().await?;

    // get postgres pool
    let db_url = env::var("DATABASE_URL")
        .unwrap_or(format!("postgresql://{}:{}@localhost:5432/{}",
                           env::var("POSTGRES_USER")
                               .map_err(|_| anyhow::anyhow!("POSTGRES_USER is not set"))?,
                           env::var("POSTGRES_PASSWORD")
                               .map_err(|_| anyhow::anyhow!("POSTGRES_PASSWORD is not set"))?,
                           env::var("POSTGRES_DB")
                               .map_err(|_| anyhow::anyhow!("POSTGRES_DB is not set"))?));
    let pool = crate::data::create_db_pool(&db_url).await?;
    crate::data::postgres::init_database(&pool).await?;

    // start api server
    tokio::spawn(async move {
        if let Err(e) = run_api_server(pool.clone()).await {
            eprintln!("[necko-xray]: API Server error: {}", e);
        }
    });

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => println!("[necko-xray]: Received SIGTERM (shutting down daemon)"),
        _ = sigint.recv() => println!("[necko-xray]: Received SIGINT (shutting down daemon)"),
    }

    cleanup();
    Ok(())
}

pub async fn start_xray() -> anyhow::Result<()> {
    if is_xray_running() {
        anyhow::bail!("Xray is already running");
    }

    let mut xray = Command::new("/usr/local/bin/xray")
        .arg("-config")
        .arg("/etc/xray/config.json")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to start Xray process");

    let xray_pid = xray.id().unwrap();
    println!("[necko-xray]: Xray started with PID: {}", xray_pid);

    std::fs::write(XRAY_PID_FILE, xray_pid.to_string())?;

    tokio::spawn(async move {
        match xray.wait().await {
            Ok(status) => println!("[necko-xray]: Xray exited with status: {}", status),
            Err(e) => eprintln!("[necko-xray]: Failed to wait for Xray: {}", e),
        }
        let _ = std::fs::remove_file(XRAY_PID_FILE);
    });

    Ok(())
}

fn cleanup() {
    let _ = std::fs::remove_file(SOCKET_PATH);
    let _ = std::fs::remove_file(XRAY_PID_FILE);
    lock::release_lock();
}

pub async fn stop() -> anyhow::Result<()> {
    let pid_str = std::fs::read_to_string(XRAY_PID_FILE)
        .map_err(|_| anyhow::anyhow!("Xray daemon is not running"))?;
    let pid: i32 = pid_str.trim().parse()?;

    println!("[necko-xray]: Stopping Xray (PID: {})...", pid);

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid), Signal::SIGTERM)?;
        println!("[necko-xray]: Stop signal sent to Xray successfully");
    }

    Ok(())
}

async fn run_api_server(pool: PgPool) -> anyhow::Result<()> {
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("[necko-xray]: API Server listening on {}", SOCKET_PATH);

    loop {
        let (mut stream, _) = listener.accept().await?;

        let pool = pool.clone();
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
                crate::api::handle_command(pool, req).await
            };

            let response = result.unwrap_or_else(|e| e.to_string());
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

pub async fn send_request(request: Request) -> anyhow::Result<String> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;

    let bytes = encode_to_vec(&request, standard())?;
    let len = (bytes.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}
