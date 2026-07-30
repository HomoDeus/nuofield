use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use nuofield_core::NewEvent;
use reqwest::{Client, Response};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(name = "nuofield", version, about = "Agent-first NuoField client")]
struct Cli {
    #[arg(
        long,
        env = "NUOFIELD_URL",
        default_value = "http://127.0.0.1:3000",
        global = true
    )]
    server: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Id,
    Append {
        #[arg(long)]
        file: PathBuf,
    },
    Workspace {
        workspace_id: String,
    },
    Events {
        workspace_id: String,
    },
    Export {
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new();
    let base = cli.server.trim_end_matches('/');

    match cli.command {
        Command::Id => println!("{}", Uuid::new_v4()),
        Command::Append { file } => {
            let content = fs::read_to_string(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let event: NewEvent = serde_json::from_str(&content)
                .with_context(|| format!("invalid event JSON in {}", file.display()))?;
            let response = client
                .post(format!("{base}/v1/events"))
                .json(&event)
                .send()
                .await
                .context("event request failed")?;
            print_response(response).await?;
        }
        Command::Workspace { workspace_id } => {
            let response = client
                .get(format!("{base}/v1/workspaces/{workspace_id}"))
                .send()
                .await
                .context("workspace request failed")?;
            print_response(response).await?;
        }
        Command::Events { workspace_id } => {
            let response = client
                .get(format!("{base}/v1/workspaces/{workspace_id}/events"))
                .send()
                .await
                .context("events request failed")?;
            print_response(response).await?;
        }
        Command::Export { output } => {
            let response = client
                .get(format!("{base}/v1/export"))
                .send()
                .await
                .context("export request failed")?;
            let status = response.status();
            let body = response.text().await.context("failed to read export")?;
            if !status.is_success() {
                bail!("server returned {status}: {body}");
            }
            if let Some(path) = output {
                fs::write(&path, format!("{body}\n"))
                    .with_context(|| format!("failed to write {}", path.display()))?;
            } else {
                println!("{body}");
            }
        }
    }

    Ok(())
}

async fn print_response(response: Response) -> Result<()> {
    let status = response.status();
    let body = response.text().await.context("failed to read response")?;
    if !status.is_success() {
        bail!("server returned {status}: {body}");
    }
    let json: serde_json::Value =
        serde_json::from_str(&body).context("server returned non-JSON")?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
