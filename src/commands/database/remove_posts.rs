use crate::{commands::ExecutableCommand, database::Database};
use anyhow::Result;
use clap::Parser;
use log::info;
use reqwest::Url;

/// Remove one or more URLs into the posted_urls table.
///
/// Useful for making the bot repost URLs that may not have been properly posted.
///
/// Please note that this does not delete the post from Bluesky itself.
#[derive(Debug, Clone, Parser)]
pub struct RemovePostsCommand {
    /// A comma-seperated list of URLs to posts.
    #[clap(value_delimiter = ',', required = true)]
    posts: Vec<Url>,

    /// The connection string to use when connecting to the sqlite database.
    /// Supports some connection parameters.
    #[arg(
        long = "database-url",
        env = "DATABASE_URL",
        default_value = format!("sqlite://{}?mode=rwc", dirs::config_local_dir().unwrap().join("skywrite").join("db.sqlite3").to_str().unwrap())
    )]
    database_url: String,
}

impl ExecutableCommand for RemovePostsCommand {
    async fn run(self) -> Result<()> {
        let database = Database::new(&self.database_url).await?;

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
