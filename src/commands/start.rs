use super::{ExecutableCommand, GlobalArguments};
use crate::bsky::{BlueskyHandler, PostData, PostEmbed};
use crate::database::Database;
use crate::rss::RssHandler;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration};
use clap::Parser;
use futures::future;
use log::{debug, error, info, warn};
use reqwest::Url;
use scraper::{Html, Selector};
use std::primitive;
use std::sync::Arc;
use tokio::time::sleep;

/// Start the bot and begin checking for new RSS posts on an interval.
#[derive(Debug, Parser)]
pub struct StartCommand {
    /// The base URL of the service to communicate with.
    ///
    /// Note that that you must delete the file at `{data-path}/agentconfig.json` to change this after it has been initially set.
    #[clap(
        required = true,
        default_value = "https://bsky.social",
        long = "app-service",
        env = "APP_SERVICE"
    )]
    service: Url,

    /// The username or email of the application's account.
    #[clap(required = true, long = "app-identifier", env = "APP_IDENTIFIER")]
    identifier: String,

    /// The app password to use for authentication.
    #[clap(required = true, long = "app-password", env = "APP_PASSWORD")]
    password: String,

    /// The interval of time in seconds between checking for new posts.
    #[clap(
        default_value_t = 300,
        long = "rerun-interval-seconds",
        env = "RERUN_INTERVAL_SECONDS"
    )]
    run_interval_seconds: u64,

    /// The number of hours in the past the bot should check for posts that haven't been posted at startup.
    /// Useful for backdating an account or when an outage occurs.
    ///
    /// It is recommended to set this to a minimum of "1" as otherwise posts may get missed.
    #[clap(
        default_value_t = 3,
        long = "rss-feed-backdate-hours",
        env = "RSS_FEED_BACKDATE_HOURS"
    )]
    rss_feed_backdate_hours: u16,

    /// A comma-seperated list of URLs pointing directly to RSS feeds.
    #[clap(
        required = true,
        long = "rss-feed-urls",
        env = "RSS_FEED_URLS",
        value_delimiter = ','
    )]
    rss_feed_urls: Vec<Url>,

    /// Whether Bluesky posts should have comments disabled.
    #[clap(
        default_value_t = true,
        long = "disable-post-comments",
        env = "DISABLE_POST_COMMENTS"
    )]
    disable_post_comments: primitive::bool,

    /// A comma-seperated list of languages in ISO-639-1 format to classify posts under.
    /// This should corrolate to the language of the posts the feed is linking to.
    #[clap(
        required = true,
        default_value = "en",
        long = "post-languages",
        env = "POST_LANGUAGES",
        value_delimiter = ','
    )]
    post_languages: Vec<String>,
}

impl ExecutableCommand for StartCommand {
    async fn run(self, global_args: GlobalArguments) -> Result<()> {
        let database = Arc::new(Database::new(&global_args.database_url).await?);
        let bsky_handler = Arc::new(
            BlueskyHandler::new(
                self.service,
                global_args.data_path,
                self.disable_post_comments,
            )
            .await?,
        );
        bsky_handler.login(&self.identifier, &self.password).await?;

        let mut handles = vec![];
        for feed in self.rss_feed_urls {
            let mut rsshandler = RssHandler::new(
                feed,
                database.clone(),
                Duration::hours(self.rss_feed_backdate_hours as i64),
            );

            handles.push(tokio::spawn({
                let database = database.clone();
                let bsky_handler = bsky_handler.clone();
                let post_languages = self.post_languages.clone();
                let og_description_selector = Selector::parse(r#"meta[property="og:description"]"#)
                    .expect("selector expression should be parseable");
                let og_image_selector = Selector::parse(r#"meta[property="og:image"]"#)
                    .expect("selector expression should be parseable");

                async move {
                    loop {
                        bsky_handler.sync_session().await.unwrap();
                        info!(
                            "Checking for unposted entries for RSS feed: {}",
                            rsshandler.get_feed()
                        );

                        if let Ok(feed) = rsshandler.fetch_unposted().await {
                            for post in feed.entries {
                                let Some(post_link) = post.links.first() else {
                                    debug!(
                                        "Post '{:?}' did not have any links attached, it will be skipped.",
                                        post.title
                                    );
                                    continue;
                                };
                                info!("Running for post '{}'", post_link.href);

                                // Fetch the page text to extract opengraph data.
                                let page = match reqwest::get(&post_link.href).await {
                                    Ok(res) => res.text().await.context("failed to page text").unwrap(),
                                    Err(err) => {
                                        error!(
                                            "Failed to get page data for {}: {err} - it will be retried next interval",
                                            &post_link.href
                                        );
                                        continue;
                                    }
                                };

                                // Synchronously obtain data from the page so that we do not carry `html` across an await point.
                                let post_data = {
                                    let html = scraper::Html::parse_document(&page);
                                    PostData {
                                        created_at: post.published.unwrap_or(DateTime::default()),
                                        text: format!(
                                            "{} - {}",
                                            post.title
                                                .clone()
                                                .map_or(String::from("New post"), |f| f.content),
                                            post_link.href
                                        ),
                                        languages: post_languages.clone(),
                                        embed: Some(PostEmbed {
                                            title: post
                                                .title
                                                .map(|f| f.content)
                                                .unwrap_or_else(|| post_link.href.clone()),
                                            description: post
                                                .summary
                                                .map(|summary| {
                                                    Html::parse_fragment(&summary.content)
                                                        .tree
                                                        .into_iter()
                                                        .filter_map(|node| {
                                                            node.as_text()
                                                                .map(|text| text.text.to_string())
                                                        })
                                                        .collect::<String>()
                                                })
                                                .or_else(|| {
                                                    html.select(&og_description_selector)
                                                        .next()
                                                        .and_then(|desc| {
                                                            desc.value()
                                                                .attr("content")
                                                                .map(|a| a.to_string())
                                                        })
                                                })
                                                .unwrap_or_else(|| {
                                                    "This site has not provided a description"
                                                        .into()
                                                }),
                                            thumbnail_url: html
                                                .select(&og_image_selector)
                                                .next()
                                                .and_then(|f| f.value().attr("content"))
                                                .and_then(|u| Url::parse(u).ok()),
                                            uri: Url::parse(&post_link.href).unwrap(),
                                        }),
                                    }
                                };

                                // Post the resulting data to Bluesky and add it to the database if successful.
                                bsky_handler.post(post_data).await.unwrap();
                                database
                                    .add_posted_url(&post.links.first().unwrap().href.to_string())
                                    .await
                                    .unwrap();
                            }

                            // Remove old posts from the database.
                            if let Err(err) = database.remove_old_stored_posts().await {
                                warn!("Failed to run query to remove old stored posts {err}");
                            }
                        } else {
                            error!(
                                "Failed to fetch feed {}: skipping for this iteration",
                                rsshandler.get_feed()
                            );
                        };

                        // Wait interval time for next iteration.
                        info!(
                            "Now waiting for {} seconds before re-running",
                            self.run_interval_seconds
                        );
                        sleep(std::time::Duration::from_secs(self.run_interval_seconds)).await;
                    }
                }
            }));
        }

        future::try_join_all(handles).await?;

        Ok(())
    }
}
