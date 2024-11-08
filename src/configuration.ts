export default class Config {
    public static readonly PERSIST_DATA_PATH = "./data/persist.json";
    public static readonly DATABASE_PATH = "./data/posts.sqlite3";

    public static getService(): string {
        const env = Deno.env.get("SERVICE");
        if (!env) {
            throw new Error("SERVICE not set");
        }
        return env;
    }

    public static getIdentifier(): string {
        const env = Deno.env.get("IDENTIFIER");
        if (!env) {
            throw new Error("IDENTIFIER not set");
        }
        return env;
    }

    public static getAppPassword(): string {
        const env = Deno.env.get("APP_PASSWORD");
        if (!env) {
            throw new Error("APP_PASSWORD not set");
        }
        return env;
    }

    public static getCronIntervalMinutes(): number {
        const env = Deno.env.get("CRON_INTERVAL_MINUTES");
        if (!env) {
            return 5;
        }
        return parseInt(env);
    }

    public static getFeedBackdateHours(): number {
        const env = Deno.env.get("FEED_FETCH_BACKDATE_HOURS");
        if (!env) {
            return 3;
        }
        return parseInt(env);
    }

    public static getRssFeeds(): string[] {
        const env = Deno.env.get("FEED_URLS");
        if (!env) {
            throw new Error("FEED_URLS not set");
        }

        const urls = env.split(",");
        urls.map((url) => url.trim()).forEach((url, idx) => {
            urls[idx] = url.trim();
            try {
                new URL(url);
            } catch (err) {
                throw new Error(`Unable to parse feed url: ${err}`);
            }
        });
        return urls;
    }
}
