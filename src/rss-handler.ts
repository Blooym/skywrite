import { datetime } from "ptera";
import { parseFeed } from "rss";
import Config from "./configuration.ts";
import { DatabaseHandler } from "./database-handler.ts";
import { Logger } from "./logger.ts";

export class RSSHandler {
    private filterDate: Date;
    private readonly feed: string;
    private readonly databaseHandler: DatabaseHandler;
    private readonly logger: Logger;

    public constructor(feed: string, idx: number, database: DatabaseHandler) {
        this.logger = new Logger(`RSSHandler${idx}`);
        this.logger.debug(`Initializing for feed ${feed}`);
        this.filterDate = datetime().subtract({
            hour: Config.getFeedBackdateHours(),
        }).toJSDate();
        this.feed = feed;
        this.databaseHandler = database;
    }

    public async fetchValidUnposted() {
        const rssData = await parseFeed(
            await (await fetch(
                this.feed,
            )).text(),
        );
        const posts = rssData.entries.filter((post) => {
            return post.published &&
                post.published > this.filterDate;
        }).filter((post) => {
            const postUrl = post.links[0].href;
            if (!postUrl) {
                return false;
            }
            return !this.databaseHandler.hasPostedUrl(postUrl);
        });
        this.filterDate = datetime().toJSDate();
        return posts;
    }

    // Thank you <3
    // https://github.com/milanmdev/bsky.rss/blob/79f097a657b73d7d692c4f9241c041c4857f75d5/app/utils/rssHandler.ts#L315
    public static stripHtml(html: string | undefined): string | undefined {
        if (!html) {
            return undefined;
        }
        return html
            ?.replace(/<\/?[^>]+(>|$)/g, " ")
            .replaceAll("&nbsp;", " ")
            .trim()
            .replace(/  +/g, " ");
    }
}
