mod bsky;
mod commands;
mod database;
mod image;
mod rss;

use anyhow::Result;
use clap::Parser;
use commands::CommandRoot;
use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();

    CommandRoot::parse().run().await
}
