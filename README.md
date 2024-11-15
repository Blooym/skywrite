# Skywrite

> [!IMPORTANT]
> This project will break release-to-release until stablised! There is no
> promise of stability or compatibility between versions until v1. You have been
> warned.

A simple RSS feed subscriber -> Bluesky post bot.

## Features

- Post multiple feeds to a single account.
- Automatically backdated posts fetched X hours from before bot startup.
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
  skywrite:
    image: ghcr.io/blooym/skywrite
    restart: unless-stopped
    environment:
      - APP_SERVICE=
      - APP_IDENTIFIER=
      - APP_PASSWORD=
      - RSS_FEED_URLS=
      - RERUN_INTERVAL_SECONDS=
      - RSS_FEED_BACKDATE_HOURS=
      - POST_LANGUAGES=
      - DISABLE_POST_COMMENTS=
    volumes:
      - skywrite-data:/opt/skywrite/data

volumes:
  skywrite-data:
```

2. Start the stack

```
docker compose up -d
```

### Manual

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed and
   in your `$PATH`.
2. Install the project binary

```
cargo install --git https://github.com/Blooym/skywrite.git
```

3. Copy `.env.example` to `.env` and fill in the values as necessary.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

4. Run the project from the same directory as `.env`

```
skywrite start
```

## Configuration

Configuration is handled entirely through environment variables or command-line
flags. The available configuration options for the 'start' command are:

- `APP_SERVICE`: The full URL to the service to communicate with. Defaults to
  `https://bsky.social`
- `APP_IDENTIFIER`: The username or email of the application's account.
- `APP_PASSWORD`: The app password to use for authentication.
- `DATA_PATH`: The base directory to store things like configuration files and
  other persistent data.
- `DATABASE_URL`: The connection string to use when connecting to the sqlite
  database. Supports some connection parameters.
- `RERUN_INTERVAL_SECONDS`: The interval of time in seconds between checking for
  new posts.
- `RSS_FEED_BACKDATE_HOURS`: The number of hours in the past the bot should
  check for posts that haven't been posted at startup. Useful for backdating an
  account or when an outage occurs.
- `RSS_FEED_URLS`: A comma-seperated list of URLs pointing directly to RSS
  feeds.
- `DISABLE_POST_COMMENTS`: Whether Bluesky posts should have comments disabled.
- `POST_LANGUAGES`: A comma-seperated list of languages in **ISO-639-1** to
  classify posts under. This should corrolate to the language of the posts the
  feed is linking to.
