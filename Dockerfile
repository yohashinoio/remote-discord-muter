# Deploy client and server

FROM node:16-alpine AS client_builder

RUN apk add --no-cache libc6-compat
WORKDIR /app
ENV NEXT_TELEMETRY_DISABLED 1
COPY ./client/web .

RUN npm ci
RUN npm run build

FROM rust:1.62.0-alpine AS server_builder

RUN apk add --no-cache gcc musl-dev
WORKDIR /app
COPY ./server .
RUN cargo build --release

FROM node:16-alpine as runner

ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1

COPY --from=client_builder /app/next.config.js next.config.js
COPY --from=client_builder /app/.next .next
COPY --from=client_builder /app/public public
COPY --from=client_builder /app/node_modules node_modules
COPY --from=client_builder /app/package.json package.json
COPY --from=client_builder /app/package-lock.json package-lock.json

COPY --from=server_builder /app/target/release/server server

CMD npm run start & ./server
