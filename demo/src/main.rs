use clap::Parser;
use demo::{Args, Command};
use reqwest::Url;

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().compact().init();

    let args = Args::parse();

    match args.cmd {
        Command::Server => demo::run_sample_server().await?,
        demo::Command::Client(_client_args) => {
            let _ = reqwest::get(Url::parse("http://localhost:5000/").expect("valid local url"))
                .await?;
        }
    }

    Ok(())
}
