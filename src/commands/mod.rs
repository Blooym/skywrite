mod database;
mod start;

use anyhow::Result;
use clap::Parser;
use database::DatabaseCommandBase;
use start::StartCommand;

pub trait ExecutableCommand {
    /// Consume the instance of and run this command.
    async fn run(self) -> Result<()>;
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about)]
pub struct CommandRoot {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Parser)]
enum Commands {
    Start(StartCommand),
    Database(DatabaseCommandBase),
}

impl ExecutableCommand for CommandRoot {
    async fn run(self) -> Result<()> {
        match self.command {
            Commands::Start(cmd) => cmd.run().await,
            Commands::Database(cmd) => cmd.run().await,
        }
    }
}
