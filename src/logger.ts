export class Logger {
    private readonly prefix: string;

    constructor(prefix: string) {
        this.prefix = prefix;
    }

    // deno-lint-ignore no-explicit-any
    public trace(...message: any[]) {
        console.trace(`TRACE: [${this.prefix}]  ${message}`);
    }

    // deno-lint-ignore no-explicit-any
    public debug(...message: any[]) {
        console.debug(`DEBUG: [${this.prefix}] ${message}`);
    }

    // deno-lint-ignore no-explicit-any
    public info(...message: any[]) {
        console.info(`INFO: [${this.prefix}] ${message}`);
    }

    // deno-lint-ignore no-explicit-any
    public warn(...message: any[]) {
        console.warn(`WARN: [${this.prefix}] ${message}`);
    }

    // deno-lint-ignore no-explicit-any
    public error(...message: any[]) {
        console.error(`ERROR: [${this.prefix}] ${message}`);
    }
}
