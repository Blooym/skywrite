# Bluesky RSS Bot

> [!IMPORTANT]
> This project will break release-to-release until stablised! There is no
> promise of stability or compatibility between versions until v1. You have been
> warned.

## Features

- Posts automatically backdated and fetched X hours from before bot startup.
- Duplicate post detection via URL stored in persistent database.
- Link embedding with image support.

## Setup

### Docker

1. Copy the following to a local file named `docker-compose.yml` or add the
   service to your existing stack and fill in the environment variables.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

```yml
services:
  bsky-rss-bot:
    image: ghcr.io/blooym/bsky-rss-bot
    restart: unless-stopped
    environment:
      - APP_SERVICE=
      - APP_IDENTIFIER=
      - APP_PASSWORD=
      - RSS_FEED_URL=
      - RERUN_INTERVAL_SECONDS=
      - RSS_FEED_BACKDATE_HOURS=
      - POSTING_LANGUAGES=
      - DISABLE_POST_COMMENTS=
    volumes:
      - bsky-rss-bot-data:/opt/bsky-rss-bot/data

volumes:
  bsky-rss-bot-data:
```

2. Start the stack

```
docker compose up -d
```

### Manual

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed and
   in your `$PATH`.
2. Clone the repository

```
git clone https://github.com/Blooym/bsky-rss-bot.git
```

3. Copy `.env.example` to `.env` and fill in the values as necessary.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

4. Run the project

```
cargo run -r
```

## Configuration

Configuration is handled entirely through environment variables, usually using
either docker directly or `.env`.

- `APP_SERVICE`: The full URL to the service to communicate with. Defaults to
  `https://bsky.social`
- `APP_IDENTITY`: The username or email of the application's account.
- `APP_PASSWORD`: The app password to use for authentication.
- `RSS_FEED_URL`: A full URL including protocol to an RSS feed.
- `RERUN_INTERVAL_SECONDS`: The interval of time in seconds between checking for
  new posts.
- `RSS_FEED_BACKDATE_HOURS`: The number of hours in the past the bot should
  check for posts that haven't been posted at startup. Useful for backdating an
  account or when an outage occurs.
- `POSTING_LANGUAGES`: A comma-seperated list of languages in **ISO-639-1** to
  classify posts under. This should corrolate to the language of the posts the
  feed is linking to.
- `DISABLE_POST_COMMENTS`: Whether Bluesky posts should have comments disabled.
