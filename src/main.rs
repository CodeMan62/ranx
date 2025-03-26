use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
mod config;
use anyhow::Result;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    #[arg(short, long, default_value = "info")]
    log_level: String,
}
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let level = match args.log_level.to_lowercase().as_str() {
        "debug" => Level::DEBUG,
        "Error" => Level::ERROR,
        "Info" => Level::INFO,
        "Trace" => Level::TRACE,
        "Warn" => Level::WARN,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Bhak bhosdike");

    let config = config::load_config(&args.config)?;
    info!("Configuration loaded successfully");

    info!("Starting ranx server");
    Ok(())
}
