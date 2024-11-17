use crate::{
    commands::{ExecutableCommand, GlobalArguments},
    database::Database,
};
use anyhow::{bail, Result};
use clap::Parser;

/// Export all posts out of the post_urls table as a comma seperated list.
#[derive(Debug, Parser)]
pub struct ExportPostsCommand;

impl ExecutableCommand for ExportPostsCommand {
    async fn run(self, global_args: GlobalArguments) -> Result<()> {
        let database = Database::new(&global_args.database_url).await?;

        let Some(posts) = database.get_all_post_urls().await? else {
            bail!("There are no posts saved in the database");
        };

        println!("{}", posts.join(","));

        Ok(())
    }
}
