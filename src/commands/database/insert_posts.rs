use crate::{
    commands::{ExecutableCommand, GlobalArguments},
    database::Database,
};
use anyhow::Result;
use clap::Parser;
use log::info;
use reqwest::Url;

/// Insert one or more URLs into the posted_urls table.
///
/// Useful for making the bot ignore URLs that may otherwise be unwantedly posted.
///
/// Please note that this does not create a new post on Bluesky.
#[derive(Debug, Parser)]
pub struct InsertPostsCommand {
    /// A comma-seperated list of URLs to posts.
    #[clap(value_delimiter = ',', required = true)]
    posts: Vec<Url>,
}

impl ExecutableCommand for InsertPostsCommand {
    async fn run(self, global_args: GlobalArguments) -> Result<()> {
        let database = Database::new(&global_args.database_url).await?;

        for post in self.posts {
            let url = post.as_str();
            if !database.has_posted_url(url).await? {
                info!("Marking {url} as already posted");
                database.add_posted_url(url).await?;
            } else {
                info!("{url} is already marked as posted");
            }
        }

        Ok(())
    }
}
