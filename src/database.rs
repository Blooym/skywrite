use anyhow::Result;
use log::debug;
use sqlx::{migrate, query, SqlitePool};

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        migrate!().run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn remove_old_stored_posts(&self) -> Result<()> {
        debug!("Removing old posted_urls entries");
        query!("DELETE FROM posted_urls WHERE ROWID IN (SELECT ROWID FROM posted_urls ORDER BY ROWID DESC LIMIT -1 OFFSET 25000)").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn add_posted_url(&self, url: &str) -> Result<()> {
        debug!("Storing {url} in posted_urls");
        query!("INSERT INTO posted_urls (url) VALUES (?)", url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_posted_url(&self, url: &str) -> Result<()> {
        debug!("Removing {url} from posted_urls");
        query!("DELETE FROM posted_urls WHERE url = ?", url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn has_posted_url(&self, url: &str) -> Result<bool> {
        debug!("Checking if {url} exists in posted_urls table");
        Ok(query!("SELECT url FROM posted_urls WHERE url = ?", url)
            .fetch_optional(&self.pool)
            .await?
            .is_some())
    }

    pub async fn get_all_post_urls(&self) -> Result<Option<Vec<String>>> {
        debug!("Fetching all urls in the posted_urls table");
        let q = query!(r#"SELECT url FROM posted_urls"#)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|f| f.url)
            .collect::<Option<Vec<String>>>();
        Ok(q)
    }
}
