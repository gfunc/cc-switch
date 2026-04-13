FROM node:20-bookworm AS web-builder

RUN sed -i 's|deb.debian.org|mirrors.aliyun.com|g; s|security.debian.org|mirrors.aliyun.com|g' /etc/apt/sources.list.d/debian.sources

WORKDIR /app
RUN corepack enable
RUN npm config set registry https://registry.npmmirror.com
RUN pnpm config set registry https://registry.npmmirror.com

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY tsconfig.json tsconfig.node.json vite.config.ts postcss.config.cjs tailwind.config.cjs components.json ./
COPY src/index.html ./index.html
COPY src ./src
COPY assets ./assets

RUN pnpm install --no-frozen-lockfile
RUN pnpm build:renderer

FROM rust:1.90-bookworm

# Build-time feature flags.
# Set to "api-only" for the lightweight Docker API service (no GTK/display needed at runtime).
# Leave empty for full desktop mode.
ARG CARGO_FEATURES=""

RUN sed -i 's|deb.debian.org|mirrors.aliyun.com|g; s|security.debian.org|mirrors.aliyun.com|g' /etc/apt/sources.list.d/debian.sources

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       pkg-config \
       libssl-dev \
       libgtk-3-dev \
       libwebkit2gtk-4.1-dev \
       libayatana-appindicator3-dev \
       librsvg2-dev \
       ca-certificates \
       xvfb \
       xauth \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /usr/local/cargo \
        && printf '%s\n' \
            '[source.crates-io]' \
            'replace-with = "aliyun"' \
            '[source.aliyun]' \
            'registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"' \
            '[net]' \
            'git-fetch-with-cli = true' \
            > /usr/local/cargo/config.toml

WORKDIR /app

COPY src-tauri/ ./src-tauri/
COPY --from=web-builder /app/dist ./web-dist

# Pass optional feature flags (e.g. api-only) if provided
RUN if [ -n "$CARGO_FEATURES" ]; then \
        cargo build --manifest-path src-tauri/Cargo.toml --release --features "$CARGO_FEATURES"; \
    else \
        cargo build --manifest-path src-tauri/Cargo.toml --release; \
    fi

EXPOSE 3001
