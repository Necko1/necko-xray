use crate::CliCommands;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::api::daemon::SOCKET_PATH;

pub async fn handle_command(cmd: CliCommands) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;

    let request = match cmd {
        CliCommands::Stats { email } => format!("stats {}", email)
    };

    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    println!("{}", response);
    Ok(())
}
