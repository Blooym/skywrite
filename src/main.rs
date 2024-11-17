mod bsky;
mod commands;
mod database;
mod rss;

use anyhow::Result;
use clap::Parser;
use commands::CommandRoot;
use dotenvy::dotenv;
use std::env;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .with_thread_ids(true)
        .init();

    CommandRoot::parse().run().await
}
