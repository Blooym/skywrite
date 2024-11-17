mod export_posts;
mod insert_posts;
mod remove_posts;

use anyhow::Result;
use clap::Parser;
use export_posts::ExportPostsCommand;
use insert_posts::InsertPostsCommand;
use remove_posts::RemovePostsCommand;

use super::{ExecutableCommand, GlobalArguments};

/// A collection of commands that perform actions to the database.
#[derive(Debug, Parser)]
pub struct DatabaseCommandBase {
    #[clap(subcommand)]
    subcommand: DatabaseSubcommand,
}

#[derive(Debug, Parser)]
enum DatabaseSubcommand {
    InsertPost(InsertPostsCommand),
    RemovePost(RemovePostsCommand),
    ExportPosts(ExportPostsCommand),
}

impl ExecutableCommand for DatabaseCommandBase {
    async fn run(self, global_args: GlobalArguments) -> Result<()> {
        match self.subcommand {
            DatabaseSubcommand::InsertPost(cmd) => cmd.run(global_args).await,
            DatabaseSubcommand::RemovePost(cmd) => cmd.run(global_args).await,
            DatabaseSubcommand::ExportPosts(cmd) => cmd.run(global_args).await,
        }
    }
}
