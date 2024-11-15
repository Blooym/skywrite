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
FROM alpine
RUN adduser -S -s /bin/false -D skywrite
USER skywrite
WORKDIR /opt/skywrite
RUN mkdir /opt/skywrite/data

ENV RUST_BACKTRACE=1
ENV DATABASE_URL=sqlite:///opt/skywrite/data/db.sqlite3?mode=rwc
ENV DATA_PATH=/opt/skywrite/data
COPY --from=builder /build/target/release/skywrite /usr/local/bin/skywrite
ENTRYPOINT ["/usr/local/bin/skywrite", "start"]