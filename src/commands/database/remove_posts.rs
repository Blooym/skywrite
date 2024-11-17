use crate::{
    commands::{ExecutableCommand, GlobalArguments},
    database::Database,
};
use anyhow::Result;
use clap::Parser;
use log::info;
use reqwest::Url;

/// Remove one or more URLs into the posted_urls table.
///
/// Useful for making the bot repost URLs that may not have been properly posted.
///
/// Please note that this does not delete the post from Bluesky itself.
#[derive(Debug, Parser)]
pub struct RemovePostsCommand {
    /// A comma-seperated list of URLs to posts.
    #[clap(value_delimiter = ',', required = true)]
    posts: Vec<Url>,
}

impl ExecutableCommand for RemovePostsCommand {
    async fn run(self, global_args: GlobalArguments) -> Result<()> {
        let database = Database::new(&global_args.database_url).await?;

        for post in self.posts {
            let url = post.as_str();
            if database.has_posted_url(url).await? {
                info!("Removing {url} from already posted list");
                database.remove_posted_url(url).await?;
            } else {
                info!("{url} is not marked as posted");
            }
        }

        Ok(())
    }
}
