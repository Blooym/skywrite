use crate::database::Database;
use anyhow::Result;
use chrono::{Duration, Utc};
use feed_rs::model::Feed;
use log::debug;
use reqwest::Url;
use std::sync::Arc;

#[derive(Debug)]
pub struct RssHandler {
    filter_date: chrono::DateTime<Utc>,
    database: Arc<Database>,
    feed_backdate_duration: Duration,
    feed: Url,
}

impl RssHandler {
    pub fn new(feed: Url, database: Arc<Database>, feed_backdate: Duration) -> Self {
        let filter_date = Utc::now() - feed_backdate;
        debug!("Initializing RSS handler for {feed} with starting filter date of {filter_date}");
        Self {
            database,
            feed,
            filter_date,
            feed_backdate_duration: feed_backdate,
        }
    }

    pub fn get_feed(&self) -> &Url {
        &self.feed
    }

    pub async fn fetch_unposted(&mut self) -> Result<Feed> {
        let content = reqwest::get(self.feed.clone()).await?.bytes().await?;
        let mut feed = feed_rs::parser::parse(&content[..])?;
        let mut new_entries = vec![];
        for item in feed.entries {
            // Only count posts that are after the filter date.
            let Some(pub_date) = item.published else {
                continue;
            };
            if pub_date <= self.filter_date {
                continue;
            }

            // Check for duplicate link. No link, no post.
            let Some(link) = item.links.first() else {
                continue;
            };
            if self.database.has_posted_url(&link.href).await? {
                continue;
            }

            new_entries.push(item);
        }
        self.filter_date = Utc::now() - self.feed_backdate_duration;
        feed.entries = new_entries;
        Ok(feed)
    }
}
