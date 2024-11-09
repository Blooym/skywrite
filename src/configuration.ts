export default class Config {
    public static readonly PERSIST_DATA_PATH = "./data/persist.json";
    public static readonly DATABASE_PATH = "./data/posts.sqlite3";

    public static getService(): string {
        const env = Deno.env.get("APP_SERVICE");
        if (env === undefined) {
            throw new Error("APP_SERVICE not set");
        }
        return env;
    }

    public static getIdentifier(): string {
        const env = Deno.env.get("APP_IDENTIFIER");
        if (env === undefined) {
            throw new Error("APP_IDENTIFIER not set");
        }
        return env;
    }

    public static getAppPassword(): string {
        const env = Deno.env.get("APP_PASSWORD");
        if (env === undefined) {
            throw new Error("APP_PASSWORD not set");
        }
        return env;
    }

    public static getCronIntervalMinutes(): number {
        const env = Deno.env.get("RSS_CRON_INTERVAL_MINUTES");
        if (env === undefined) {
            return 5;
        }
        return parseInt(env);
    }

    public static getFeedBackdateHours(): number {
        const env = Deno.env.get("RSS_FEED_FETCH_PAST_HOURS");
        if (env === undefined) {
            return 3;
        }
        return parseInt(env);
    }

    public static getDisablePostComments(): boolean {
        const env = Deno.env.get("DISABLE_POST_COMMENTS");
        if (env === undefined) {
            return true;
        }
        return JSON.parse(env);
    }

    public static getRssFeeds(): string[] {
        const env = Deno.env.get("RSS_FEED_URLS");
        if (env === undefined) {
            throw new Error("RSS_FEED_URLS not set");
        }

        const urls = env.split(",");
        urls.forEach((url, idx) => {
            urls[idx] = url.trim();
            try {
                new URL(url);
            } catch (err) {
                throw new Error(`Unable to parse feed url: ${err}`);
            }
        });
        return urls;
    }

    public static getPostLanguages(): string[] | undefined {
        const env = Deno.env.get("POSTING_LANGUAGES");
        if (env === undefined) {
            return undefined;
        }
        const langs = env.split(",");
        langs.forEach((url, idx) => {
            langs[idx] = url.trim();
        });
        return langs;
    }
}
