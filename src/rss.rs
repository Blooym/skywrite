use crate::database::Database;
use anyhow::{Result, bail};
use chrono::{DateTime, Duration, Utc};
use feed_rs::model::Feed;
use log::debug;
use reqwest::{Client, Url};
use std::sync::Arc;

pub struct RssHandler {
    client: Arc<Client>,
    database: Arc<Database>,
    feed_url: Url,
    backfill_window: Duration,
    fetch_after_date: DateTime<Utc>,
}

impl RssHandler {
    pub fn new(
        feed_url: Url,
        backfill_window: Duration,
        database: Arc<Database>,
        reqwest_client: Arc<Client>,
    ) -> Self {
        let filter_date = Utc::now() - backfill_window;
        debug!(
            "Initializing RSS handler for {feed_url} with starting date of {filter_date} (backfill window: {backfill_window})"
        );
        Self {
            client: reqwest_client,
            database,
            feed_url,
            fetch_after_date: filter_date,
            backfill_window,
        }
    }

    pub fn feed_url(&self) -> &Url {
        &self.feed_url
    }

    pub async fn fetch_unposted(&mut self) -> Result<Feed> {
        let content = {
            let response = self.client.get(self.feed_url.as_ref()).send().await?;
            if !response.status().is_success() {
                bail!(
                    "got unsuccessful status code when requesting feed {}: {}",
                    self.feed_url,
                    response.status()
                )
            }
            response.bytes().await?
        };

        let mut feed = feed_rs::parser::parse(&content[..])?;
        let mut new_entries = vec![];
        for mut item in feed.entries {
            // Only count posts that are after the filter date.
            let Some(pub_date) = item.published else {
                continue;
            };
            if pub_date <= self.fetch_after_date {
                continue;
            }

            // Prefer the first post link that is from the same domain as the rss feed.
            item.links.sort_by_key(|link| {
                Url::parse(&link.href)
                    .ok()
                    .map(|url| match (url.domain(), self.feed_url.domain()) {
                        (Some(link_domain), Some(feed_domain)) => link_domain == feed_domain,
                        _ => false,
                    })
                    .unwrap_or(false)
            });
            item.links.reverse();

            // Get the first link, if any
            let Some(link) = item.links.first() else {
                continue;
            };
            if self.database.has_posted_url(&link.href).await? {
                continue;
            }
            new_entries.push(item);
        }
        self.fetch_after_date = Utc::now() - self.backfill_window;
        feed.entries = new_entries;
        Ok(feed)
    }
}
