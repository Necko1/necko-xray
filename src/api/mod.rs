use necko_xray::Client;

pub mod daemon;

pub async fn handle_command(cmd: &str) -> anyhow::Result<String> {
    match cmd.trim() {
        cmd if cmd.starts_with("stats") => {
            let email = &cmd["stats".len()..].trim();
            stats(email).await
        },
        _ => Err(anyhow::anyhow!("Unknown command")),
    }
}

async fn stats(email: &str) -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.user_online_ip_list(email).await?;

    let formatted = response
        .iter()
        .map(|(ip, time)| format!("{}: {}", ip, time))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(formatted)
}