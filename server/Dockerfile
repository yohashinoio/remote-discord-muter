FROM rust:1.62.0-alpine AS builder

RUN apk add --no-cache gcc musl-dev
WORKDIR /work
COPY . .
RUN cargo build --release

FROM scratch

COPY --from=builder /work/target/release/server /usr/local/
CMD ["/usr/local/server"]
