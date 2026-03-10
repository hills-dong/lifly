# Backend build
FROM rust:1.94 AS backend
WORKDIR /app
COPY server/ ./server/
WORKDIR /app/server
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Frontend build
FROM node:20 AS frontend
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@latest --activate
COPY web/package.json web/pnpm-lock.yaml* ./
RUN pnpm install --frozen-lockfile || pnpm install
COPY web/ .
RUN pnpm build

# Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/server/target/release/lifly-server /usr/local/bin/
COPY --from=backend /app/server/migrations /srv/migrations
COPY --from=frontend /app/dist /srv/web
ENV STATIC_FILES_PATH=/srv/web
EXPOSE 8080
CMD ["lifly-server"]
