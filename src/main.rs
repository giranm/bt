use anyhow::{Context, Result};
use braintrust_sdk_rust::BraintrustClient;
use clap::{Args, Parser, Subcommand};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(name = "bt", about = "Braintrust CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Sql(CLIArgs<SqlArgs>),
}

#[derive(Debug, Clone, Args)]
struct BaseArgs {
    /// Output as JSON
    #[arg(short = 'j', long)]
    json: bool,

    /// Override active project
    #[arg(short = 'p', long, env = "BRAINTRUST_DEFAULT_PROJECT")]
    project: Option<String>,

    /// Override stored API key (or via BRAINTRUST_API_KEY)
    #[arg(long, env = "BRAINTRUST_API_KEY")]
    api_key: Option<String>,

    /// Override API URL (or via BRAINTRUST_API_URL)
    #[arg(long, env = "BRAINTRUST_API_URL")]
    api_url: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct CLIArgs<T: Args> {
    #[command(flatten)]
    base: BaseArgs,

    #[command(flatten)]
    args: T,
}

#[derive(Debug, Clone, Args)]
struct SqlArgs {
    /// SQL query to execute
    query: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sql(cmd) => run_sql(cmd.base, cmd.args).await?,
    }

    Ok(())
}

async fn run_sql(base: BaseArgs, args: SqlArgs) -> Result<()> {
    let mut builder = BraintrustClient::builder().blocking_login(true);
    if let Some(api_key) = &base.api_key {
        builder = builder.api_key(api_key);
    }
    if let Some(api_url) = &base.api_url {
        builder = builder.api_url(api_url);
    }
    if let Some(project) = &base.project {
        builder = builder.default_project(project);
    }

    let client = builder.build().await?;
    let login = client.wait_for_login().await?;

    let api_url = login
        .api_url
        .clone()
        .or(base.api_url)
        .unwrap_or_else(|| "https://api.braintrust.dev".to_string());
    let url = format!("{}/btql", api_url.trim_end_matches('/'));

    let request_body = json!({
        "query": args.query,
        "fmt": "json",
    });

    let http = reqwest::Client::builder()
        .build()
        .context("failed to build HTTP client")?;

    let mut request = http
        .post(url)
        .bearer_auth(&login.api_key)
        .header("Content-Type", "application/json")
        .json(&request_body);

    if !login.org_name.is_empty() {
        request = request.header("x-bt-org-name", &login.org_name);
    }

    let response = request.send().await.context("btql request failed")?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("btql request failed ({}): {}", status, body);
    }

    let payload: serde_json::Value = response
        .json()
        .await
        .context("failed to parse btql response")?;

    if base.json {
        println!("{}", serde_json::to_string(&payload)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    }

    Ok(())
}
