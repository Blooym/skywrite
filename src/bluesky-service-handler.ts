import { AtpAgent, AtpSessionData, AtUri, RichText } from "atproto-api";
import { Buffer } from "node-buffer";
import Config from "./configuration.ts";
import { Logger } from "./logger.ts";

/**
 * Post data that is processed when creating a bluesky post.
 */
export interface IPost {
    content: string;
    createdAt: string;
    languages?: string[];
    embed?: IEmbedData;
}

/**
 * Embed data that is processed when creating a bluesky post.
 */
export interface IEmbedData {
    uri: string;
    title: string;
    description: string;
    image_buffer?: Buffer | undefined;
}

/**
 * Wrapper for `@atproto/api` that abstracts key functionality.
 */
export class BlueskyServiceHandler {
    private readonly Logger = new Logger("BlueskyServiceHandler");
    private agpAgent: AtpAgent;

    public constructor(service: string) {
        this.Logger.debug(`Initializing with service URL ${service}`);
        this.agpAgent = new AtpAgent({
            service: service,
            persistSession: async (_evt, sess?) => {
                if (!sess) return;
                await this.writePersistSession(sess);
            },
        });
    }

    private async writePersistSession(session: AtpSessionData) {
        this.Logger.debug("Writing persisted session to disk");
        await Deno.writeTextFile(
            Config.PERSIST_DATA_PATH,
            JSON.stringify(session),
        );
    }

    private async readPersistSession(): Promise<AtpSessionData | undefined> {
        this.Logger.debug("Attempting to read persisted session from disk");
        try {
            return JSON.parse(
                await Deno.readTextFile(
                    Config.PERSIST_DATA_PATH,
                ),
            ) as AtpSessionData;
        } catch {
            return undefined;
        }
    }

    public async login(identifier: string, password: string) {
        const persistedSessionData = await this.readPersistSession();

        try {
            if (!persistedSessionData || !persistedSessionData.accessJwt) {
                throw new Error(
                    "No persisted session. Logging in with credentials.",
                );
            }

            const session = await this.agpAgent.resumeSession(
                persistedSessionData,
            );
            if (session.success) {
                this.Logger.info(
                    `Successful session resume for ${session.data.handle}`,
                );
                return session;
            } else {
                throw new Error(
                    "Authentication failure with persisted session",
                );
            }
        } catch {
            const loginData = await this.agpAgent.login({
                identifier,
                password,
            });
            if (!loginData.success) {
                throw new Error("Authentication failure with login/password");
            }
            this.Logger.info(
                `Successful login for ${loginData.data.handle}`,
            );

            return loginData;
        }
    }

    public async post(data: IPost) {
        this.agpAgent.assertAuthenticated();

        const bskyText = new RichText({ text: data.content });
        await bskyText.detectFacets(this.agpAgent);

        // Prepare bsky embed.
        let bskyEmbed;
        if (data.embed) {
            if (data.embed.image_buffer) {
                // Upload the image blob data before making the embed.
                const embedImage = await this.agpAgent.uploadBlob(
                    data.embed.image_buffer,
                    {
                        encoding: "image/jpeg",
                    },
                );
                if (!embedImage.success) {
                    throw new Error("Unable to upload image blob to bluesky");
                }
                this.Logger.info(
                    `Uploaded blob for post at uri ${data.embed.uri}`,
                );

                bskyEmbed = {
                    $type: "app.bsky.embed.external",
                    external: {
                        uri: data.embed.uri,
                        title: data.embed.title,
                        description: data.embed.description,
                        thumb: embedImage.data.blob,
                    },
                };
            } else {
                bskyEmbed = {
                    $type: "app.bsky.embed.external",
                    external: {
                        uri: data.embed.uri,
                        title: data.embed.title,
                        description: data.embed.description,
                    },
                };
            }
        }

        // Create post.
        const post = await this.agpAgent.post({
            $type: "app.bsky.feed.post",
            text: bskyText.text,
            facets: bskyText.facets,
            langs: data.languages,
            embed: bskyEmbed,
            createdAt: data.createdAt,
        });
        try {
            // Turn off replies. I don't want to handle ever dealing with
            // anything happening in a reply section.
            this.agpAgent.com.atproto.repo.createRecord({
                repo: this.agpAgent.session!.did,
                collection: "app.bsky.feed.threadgate",
                rkey: new AtUri(post.uri).rkey,
                record: {
                    $type: "app.bsky.feed.threadgate",
                    post: post.uri,
                    // empty allow-list, nobody can reply
                    allow: [],
                    createdAt: new Date().toISOString(),
                },
            });
        } catch {
            this.Logger.warn(
                `Failed to set threadgate reply restriction for ${post.uri} - continuing.`,
            );
        }
        this.Logger.info(
            `Successfully created post for RSS URI ${data.embed?.uri}: ${post.uri}`,
        );

        return post;
    }
}
