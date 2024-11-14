# ----------
#    USER
# ----------
FROM alpine:latest AS user
RUN adduser -S -s /bin/false -D bskybot
RUN mkdir /data

###########
# Builder #
###########
FROM rust:alpine AS builder 
WORKDIR /build

# Install build dependencies
RUN apk add --update build-base cmake libressl-dev

# Pre-cache dependencies
COPY ["Cargo.toml", "Cargo.lock", "./"]
RUN mkdir src \
    && echo "// Placeholder" > src/lib.rs \
    && cargo build --release \
    && rm src/lib.rs

# Build
ARG SQLX_OFFLINE true
COPY ./migrations ./migrations
COPY ./.sqlx ./.sqlx
COPY ["./src", "./src"]
RUN cargo build --release

###########
# Runtime #
###########
FROM scratch
WORKDIR /opt/skywrite
ENV RUST_BACKTRACE=1
ENV DATABASE_URL=DATABASE_URL=sqlite://./data/db.sqlite3?mode=rwc
ENV AGENT_CONFIG_PATH=./data/agentconfig.json

# Import and switch to non-root user.
COPY --from=user /data /opt/skywrite/data
COPY --from=user /etc/passwd /etc/passwd
COPY --from=user /bin/false /bin/false
USER bskybot

# Copy binary and run
COPY --from=builder /build/target/release/skywrite /usr/local/bin/skywrite
ENTRYPOINT ["/usr/local/bin/skywrite", "start"]