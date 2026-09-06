FROM node:22-alpine AS frontend
WORKDIR /build
COPY package.json package-lock.json* ./
RUN npm ci
COPY frontend ./frontend
COPY vite.config.ts vitest.config.ts ./
RUN npm run build

FROM rust:1-alpine AS backend
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY migrations ./migrations
COPY src ./src
ARG BUILD_SHA=dev
ENV BUILD_SHA=$BUILD_SHA
RUN cargo build --locked --release

FROM alpine:3.21
RUN apk add --no-cache ca-certificates libgcc && addgroup -S app && adduser -S -G app app
WORKDIR /app
COPY --from=backend /build/target/release/scene-cues /usr/local/bin/scene-cues
COPY --from=frontend /build/dist ./dist
RUN mkdir /data && chown app:app /data
USER app
EXPOSE 8080
VOLUME ["/data"]
CMD ["scene-cues"]
