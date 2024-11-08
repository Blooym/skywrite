# Bluesky RSS Bot

A minimal Bluesky RSS feed subscriber/posting bot.

## Features

- Can follow multiple feeds and post to the same account.
- Customizable backdate post fetch support
- Duplicate detection via URL.
- Link embedding support

## Setup

### Docker

_(This guide will be improved on)_

The recommended way to run this project is via the provided Dockerfile. Simply
copy `.env.example` to `.env` and fill in the values. Next, run the Dockerfile
and make sure to make a volume at `/opt/bsky-rss-poster/data` to ensure all
long-term data can be stored across restarts.

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
