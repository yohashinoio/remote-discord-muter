# Build client (to static site)
FROM node:16-alpine AS client_builder

RUN apk add --no-cache libc6-compat
WORKDIR /app
ENV NEXT_TELEMETRY_DISABLED 1
COPY ./client/web .

RUN npm ci
RUN npm run export

# Build server
FROM rust:1.62.0-alpine AS server_builder

RUN apk add --no-cache gcc musl-dev
WORKDIR /app
COPY ./server .

RUN cargo build --release

# Run
FROM node:16-alpine as runner

COPY --from=client_builder /app/out client

COPY --from=server_builder /app/target/release/server server

CMD ./server
