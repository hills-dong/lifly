# Backend build
FROM rust:1.94 AS backend
WORKDIR /app
COPY server/ ./server/
WORKDIR /app/server
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Frontend build (main web app, served at /).
# Pin pnpm to 9.x: pnpm 10's default supply-chain gates (minimum-release-age,
# hard-fail on unbuilt deps) break non-interactive builds from a committed lockfile.
FROM node:22 AS frontend
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@9.15.9 --activate
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml* ./
RUN pnpm install --frozen-lockfile || pnpm install
COPY web/ .
RUN pnpm build

# Admin frontend build (ops panel, served under /admin)
FROM node:22 AS admin
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@9.15.9 --activate
COPY admin/package.json admin/pnpm-lock.yaml* admin/pnpm-workspace.yaml* ./
RUN pnpm install --frozen-lockfile || pnpm install
COPY admin/ .
ENV VITE_BASE=/admin/
RUN pnpm build

# Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/server/target/release/lifly-server /usr/local/bin/
COPY --from=backend /app/server/migrations /srv/migrations
COPY --from=frontend /app/dist /srv/web
COPY --from=admin /app/dist /srv/admin
ENV STATIC_DIR=/srv/web
ENV ADMIN_STATIC_DIR=/srv/admin
EXPOSE 8080
CMD ["lifly-server"]
