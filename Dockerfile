FROM denoland/deno
USER deno
WORKDIR /opt/bsky-rss-bot

# Precache deps.
COPY ["deno.json", "deno.lock", "."]
COPY ./src ./src
RUN mkdir ./data
RUN deno cache src/main.ts

CMD ["run", "--unstable-cron", "--allow-read", "--allow-env", "--allow-ffi", "--allow-write", "--allow-net", "src/main.ts"]