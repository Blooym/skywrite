import { Database } from "sqlite3";
import Config from "./configuration.ts";
import { Logger } from "./logger.ts";

export class DatabaseHandler {
    private readonly database: Database;
    private readonly logger: Logger = new Logger("DatabaseHandler");

    public constructor() {
        this.logger.debug(`Initialising database at ${Config.DATABASE_PATH}`);
        this.database = new Database(Config.DATABASE_PATH);
        this.database.prepare(
            "CREATE TABLE IF NOT EXISTS posted_urls (url TEXT PRIMARY KEY)",
        ).run();
    }

    public removeOldStoredPosts() {
        try {
            this.database.prepare(
                "DELETE FROM posted_urls WHERE ROWID IN (SELECT ROWID FROM posted_urls ORDER BY ROWID DESC LIMIT -1 OFFSET 100)",
            ).run();
            this.logger.debug(
                `Successfully cleaned up older posts from the DB.`,
            );
        } catch (e) {
            this.logger.error("Failed to cleanup older posts from DB", e);
        }
    }

    public addPostedUrl(url: string): void {
        this.database.prepare(`INSERT INTO posted_urls (url) VALUES (?)`).run(
            url,
        );
        this.logger.debug(`Inserting ${url} into posted_urls table.`);
    }

    public hasPostedUrl(url: string): boolean {
        const has = this.database.prepare(
            "SELECT url FROM posted_urls WHERE url = ?",
        ).value(url) !== undefined;
        this.logger.debug(`${url} has already been posted in database: ${has}`);
        return has;
    }
}
