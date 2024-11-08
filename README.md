# Bluesky RSS Bot

A minimal Bluesky RSS feed subscriber/posting bot.

## Features

- Follow multiple feeds and post to the same account.
- Customizable post backdate fetching & posting.
- Duplicate detection via URL.
- Link embedding with images.

## Setup

### Docker

1. Clone the repository

```
git clone https://github.com/Blooym/bsky-rss-bot.git
```

2. Modify `docker-compose.yml` and fill in the environment variables.
3. Build & start the container

```
docker compose up -d
```

### Manual

1. Ensure you have [Deno](https://deno.land) installed and in your `$PATH`.
2. Clone the repository

```
git clone https://github.com/Blooym/bsky-rss-bot.git
```

3. Copy `.env.example` to `.env` and fill in the values as necessary.

4. Run the project

```
deno run --unstable-cron --allow-read --allow-env --allow-ffi --allow-write --allow-net src/main.ts
```
