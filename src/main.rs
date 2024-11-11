mod bsky;
mod database;
mod rss;

use anyhow::Result;
use bsky::{BlueskyHandler, PostData, PostEmbed};
use chrono::DateTime;
use clap::Parser;
use database::Database;
use dotenvy::dotenv;
use log::info;
use reqwest::Url;
use rss::RSSHandler;
use scraper::{Html, Selector};
use std::{path::PathBuf, primitive, sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about)]
struct Configuration {
    /// The base URL of the service to communicate with.
    ///
    /// Note that that you must delete the file at `--agent-config-path` to change this after it has been initially set.
    #[clap(
        required = true,
        default_value = "https://bsky.social",
        long = "app-service",
        env = "APP_SERVICE"
    )]
    service: Url,

    #[clap(required = true, long = "app-identity", env = "APP_IDENTITY")]
    identity: String,

    #[clap(required = true, long = "app-password", env = "APP_PASSWORD")]
    password: String,

    #[clap(
        default_value_t = 300,
        long = "cron-interval-seconds",
        env = "CRON_INTERVAL_SECONDS"
    )]
    run_interval_seconds: u64,

    #[clap(
        default_value_t = 3,
        long = "feed-backdate-hours",
        env = "RSS_FEED_FETCH_PAST_HOURS"
    )]
    feed_backdate_hours: u16,

    #[clap(
        default_value_t = true,
        long = "disable-comments",
        env = "DISABLE_COMMENTS"
    )]
    disable_comments: primitive::bool,

    #[clap(required = true, long = "rss-feed-url", env = "RSS_FEED_URL")]
    rss_feed_url: Url,

    #[clap(
        required = true,
        default_value = "en",
        long = "post-languages",
        env = "POST_LANGUAGES"
    )]
    post_languages: Vec<String>,

    // The connection string, path, or URI to the database that should connected to.
    /// Supports some connection parameters.
    #[arg(
        long = "database-url",
        env = "DATABASE_URL",
        default_value = "sqlite://./data/posts.sqlite3?mode=rwc"
    )]
    database_url: String,

    #[arg(
        long = "agent-config-path",
        env = "AGENT_CONFIG_PATH",
        default_value = "./data/config.json"
    )]
    agent_config_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Configuration::parse();

    let database = Arc::new(Database::new(&args.database_url).await?);
    let bsky_handler =
        BlueskyHandler::new(args.service, args.agent_config_path, args.disable_comments).await?;
    bsky_handler.login(&args.identity, &args.password).await?;

    let mut rsshandler = RSSHandler::new(
        args.rss_feed_url,
        Arc::clone(&database),
        args.feed_backdate_hours,
    );

    let og_description_selector = Selector::parse(r#"meta[property="og:description"]"#)
        .expect("selector expression should be parseable");
    let og_image_selector = Selector::parse(r#"meta[property="og:image"]"#)
        .expect("selector expression should be parseable");

    loop {
        info!(
            "Checking for unposted entries for RSS feed: {}",
            rsshandler.get_feed()
        );

        let posts = rsshandler.fetch_unposted().await?.entries;
        for post in &posts {
            if let Some(post_link) = post.links.first() {
                info!("Running for post '{}'", post_link.href);

                let page = reqwest::get(&post_link.href).await?.text().await?;
                let html = scraper::Html::parse_document(&page);

                bsky_handler
                    .post(PostData {
                        created_at: post.published.unwrap_or(DateTime::default()),
                        text: format!(
                            "{} - {}",
                            post.title
                                .clone()
                                .map_or(String::from("New post"), |f| f.content),
                            post_link.href
                        ),
                        languages: args.post_languages.clone(),
                        embed: Some(PostEmbed {
                            title: post
                                .title
                                .clone()
                                .map_or(post_link.href.clone(), |f| f.content),
                            description: post
                                .summary
                                .clone()
                                .map_or_else(
                                    || {
                                        html.select(&og_description_selector).next().and_then(
                                            |desc| {
                                                desc.value().attr("content").map(|a| a.to_string())
                                            },
                                        )
                                    },
                                    |summary| {
                                        let frag = Html::parse_fragment(&summary.content);
                                        let mut sentence = "".to_owned();
                                        for node in frag.tree {
                                            if let scraper::node::Node::Text(text) = node {
                                                sentence.push_str(&text.text);
                                            }
                                        }
                                        Some(sentence)
                                    },
                                )
                                .unwrap_or(String::from(
                                    "This site has not provided a description",
                                )),
                            thumbnail_url: html.select(&og_image_selector).next().and_then(|f| {
                                if let Some(a) = f.value().attr("content") {
                                    if let Ok(url) = Url::parse(a) {
                                        return Some(url);
                                    }
                                    return None;
                                }
                                None
                            }),
                            uri: Url::parse(&post_link.href)?,
                        }),
                    })
                    .await?;

                database
                    .add_posted_url(&post.links.first().unwrap().href)
                    .await?;
            } else {
                continue;
            }
        }
        database.remove_old_stored_posts().await?;
        info!(
            "Now waiting for {} seconds before re-running",
            args.run_interval_seconds
        );
        sleep(Duration::from_secs(args.run_interval_seconds)).await;
    }
}
