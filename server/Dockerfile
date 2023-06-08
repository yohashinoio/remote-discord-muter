FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build --release

EXPOSE 3000

ENTRYPOINT ["./target/release/server"]
