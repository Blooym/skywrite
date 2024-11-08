import "jsr:@std/dotenv/load";
import { parsedMeta } from "ogp-parser";
import sharp from "sharp";
import { BlueskyServiceHandler } from "./bluesky-service-handler.ts";
import Config from "./configuration.ts";
import { DatabaseHandler } from "./database-handler.ts";
import { Logger } from "./logger.ts";
import { RSSHandler } from "./rss-handler.ts";

const logger = new Logger("Main");

async function main() {
  logger.info("Starting up");

  const database = new DatabaseHandler();
  const bsky = new BlueskyServiceHandler(Config.getService());
  await bsky.login(
    Config.getIdentifier(),
    Config.getAppPassword(),
  );

  Config.getRssFeeds().forEach((feed, idx) => {
    const rssHandler = new RSSHandler(feed, idx, database);

    // Run a cron job for every feed.
    logger.info(`[rss-cron-${idx}] Scheduling cron job`);
    Deno.cron(`rss-cron-${idx}`, {
      minute: { every: Config.getCronIntervalMinutes() },
    }, async () => {
      logger.debug(`[rss-cron-${idx}] Running as scheduled`);
      const rssPosts = await rssHandler.fetchValidUnposted();
      if (rssPosts.length === 0) {
        logger.debug(`[rss-cron-${idx}] Nothing to post`);
        return;
      }

      // Attempt to post each entry.
      rssPosts.forEach(async (post) => {
        const postUrl = post.links[0].href;
        if (!postUrl) {
          return;
        }
        logger.debug(`[rss-cron-${idx}] ${postUrl}: Starting post creation`);

        // Fetch opengraph and convert to jpeg if image is available.
        const meta = await parsedMeta(postUrl);
        const image = meta.open_graph.image;
        let bytes;
        if (image) {
          logger.debug(
            `[rss-cron-${idx}] ${postUrl}: Has image in opengraph data, downloading and converting to jpeg buffer.`,
          );
          bytes = await sharp(await (await fetch(image)).arrayBuffer())
            .resize(800, null)
            .jpeg()
            .toBuffer();
        }

        // Post to bluesky with backdated date if available, current date if not.
        await bsky.post({
          content: `${post.title?.value} - ${postUrl}`,
          createdAt: post.published?.toISOString() ?? new Date().toISOString(),
          languages: Config.getPostLanguages(),
          embed: {
            title: post.title?.value ?? postUrl,
            description: post.description?.value ?? meta.description ??
              post.content?.value ?? "",
            image_buffer: bytes,
            uri: postUrl,
          },
        });

        logger.info(
          `[rss-cron-${idx}] ${postUrl}: Successfully posted to Bluesky`,
        );
        database.addPostedUrl(postUrl);
      });

      // Cleanup
      database.removeOldStoredPosts();
      logger.debug(`[rss-cron-${idx}] Finished cron run`);
    });
  });
}

if (import.meta.main) {
  await Deno.mkdir("./data", { recursive: true });
  await main();
}
