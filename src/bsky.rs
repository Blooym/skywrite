use anyhow::{Context, Result};
use bsky_sdk::{
    agent::config::{Config, FileStore},
    api::{
        app::bsky::{
            embed::external::{ExternalData, MainData},
            feed::{
                post::{self, RecordEmbedRefs},
                threadgate,
            },
        },
        types::{
            string::{Datetime, Language},
            Union,
        },
    },
    rich_text::RichText,
    BskyAgent,
};
use chrono::{DateTime, Utc};
use image::{imageops::FilterType, ImageFormat, ImageReader};
use log::{debug, info};
use reqwest::Url;
use std::{fs::create_dir_all, io::Cursor, path::PathBuf, str::FromStr};

#[derive(Debug)]
pub struct PostData {
    pub text: String,
    pub languages: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub embed: Option<PostEmbed>,
}
#[derive(Debug)]
pub struct PostEmbed {
    pub title: String,
    pub description: String,
    pub uri: Url,
    pub thumbnail_url: Option<Url>,
}

pub struct BlueskyHandler {
    pub agent: BskyAgent,
    pub config_path: PathBuf,
    pub disable_comments: bool,
}

impl BlueskyHandler {
    pub async fn new(
        service: Url,
        agent_config_path: PathBuf,
        disable_comments: bool,
    ) -> Result<Self> {
        let _ = create_dir_all(
            agent_config_path
                .parent()
                .context("invalid path for agent configuration (cannot find parent)")?,
        );
        let config = Config::load(&FileStore::new(&agent_config_path))
            .await
            .unwrap_or_else(|_| Config {
                endpoint: service
                    .to_string()
                    .strip_suffix("/")
                    .map_or(service.to_string(), |s| s.to_string()),
                ..Default::default()
            });
        Ok(Self {
            agent: BskyAgent::builder().config(config).build().await?,
            config_path: agent_config_path,
            disable_comments,
        })
    }

    pub async fn login(&self, identifier: &str, password: &str) -> Result<()> {
        self.agent.login(identifier, password).await?;
        self.agent
            .to_config()
            .await
            .save(&FileStore::new(&self.config_path))
            .await?;

        Ok(())
    }

    pub async fn post(&self, post: PostData) -> Result<()> {
        info!("Constructing post data for: '{}'", &post.text);
        let rt = RichText::new_with_detect_facets(&post.text).await?;
        let embed = match post.embed {
            Some(data) => Some(
                self.embed_external(
                    &data.title,
                    &data.description,
                    data.uri.as_ref(),
                    data.thumbnail_url,
                )
                .await
                .unwrap(),
            ),
            None => None,
        };

        info!("Creating post record for: '{}'", &post.text);
        let record = self
            .agent
            .create_record(post::RecordData {
                created_at: Datetime::from_str(&post.created_at.fixed_offset().to_rfc3339())?,
                embed,
                entities: None,
                facets: rt.facets,
                labels: None,
                langs: Some(
                    post.languages
                        .iter()
                        .map(|f| Language::from_str(f).unwrap())
                        .collect(),
                ),
                reply: None,
                tags: None,
                text: post.text,
            })
            .await?;

        if self.disable_comments {
            info!(
                "Disabling post comments via threadgate for '{}'",
                record.uri
            );
            self.agent
                .create_record(threadgate::RecordData {
                    allow: None,
                    created_at: Datetime::now(),
                    hidden_replies: None,
                    post: record.uri.clone(),
                })
                .await?;
        };

        Ok(())
    }

    pub async fn embed_external(
        &self,
        title: &str,
        description: &str,
        uri: &str,
        thumbnail_url: Option<Url>,
    ) -> Result<Union<RecordEmbedRefs>> {
        info!("Constructing external embed data for: '{uri}'");
        let thumb = if let Some(data) = thumbnail_url {
            let image_bytes = reqwest::get(data).await?.bytes().await?;
            let mut buf: Vec<u8> = vec![];
            ImageReader::new(Cursor::new(image_bytes))
                .with_guessed_format()?
                .decode()?
                .resize(800, 800, FilterType::Triangle)
                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)?;
            debug!("Uploading blob data for '{uri}'");
            let output = self.agent.api.com.atproto.repo.upload_blob(buf).await?;
            Some(output.data.blob)
        } else {
            None
        };
        Ok(Union::Refs(RecordEmbedRefs::AppBskyEmbedExternalMain(
            Box::new(
                MainData {
                    external: ExternalData {
                        description: description.into(),
                        title: title.into(),
                        uri: uri.into(),
                        thumb,
                    }
                    .into(),
                }
                .into(),
            ),
        )))
    }
}
