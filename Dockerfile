ARG RUST_VERSION=1.72.1
ARG NODE_VERSION=18
ARG APP_NAME=blog
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

# Build the application.
# Does a mount for caching, very usefull, a lot more on rust.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/blog
EOF

FROM node:${NODE_VERSION} as node_build

COPY src src
COPY assets assets
COPY tailwind.config.js .
RUN npx tailwindcss -i ./assets/css/input.css -o ./dist/output.css

FROM debian:bullseye-slim AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build ./bin/blog blog
COPY --from=node_build dist dist
COPY --from=node_build assets assets

EXPOSE 8080

CMD ["./blog"]
