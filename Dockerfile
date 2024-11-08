FROM denoland/deno
RUN apt-get update && apt-get install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*
USER deno
WORKDIR /opt/bsky-rss-poster
# Override deno's precompiled libsqlite3 with the system lib.
ENV DENO_SQLITE_PATH=/usr/lib/x86_64-linux-gnu/libsqlite3.so.0 

# Precache deps.
COPY ["deno.json", "deno.lock", "."]
COPY ./src ./src
RUN mkdir ./data
RUN deno cache src/main.ts

CMD ["run", "--unstable-cron", "--allow-all", "src/main.ts"]