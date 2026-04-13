FROM node:20-bookworm AS builder

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

FROM nginx:1.27-alpine

COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
