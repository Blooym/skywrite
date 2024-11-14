mod insert_posts;
mod remove_posts;

use anyhow::Result;
use clap::Parser;
use insert_posts::InsertPostsCommand;
use remove_posts::RemovePostsCommand;

use super::ExecutableCommand;

/// A collection of commands that perform actions to the database.
#[derive(Debug, Clone, Parser)]
pub struct DatabaseCommandBase {
    #[clap(subcommand)]
    subcommand: DatabaseSubcommand,
}

#[derive(Debug, Clone, Parser)]
enum DatabaseSubcommand {
    InsertPost(InsertPostsCommand),
    RemovePost(RemovePostsCommand),
}

impl ExecutableCommand for DatabaseCommandBase {
    async fn run(self) -> Result<()> {
        match self.subcommand {
            DatabaseSubcommand::InsertPost(cmd) => cmd.run().await,
            DatabaseSubcommand::RemovePost(cmd) => cmd.run().await,
        }
    }
}
