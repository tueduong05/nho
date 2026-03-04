FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src ./src
RUN touch ./src/main.rs
RUN cargo build --release
RUN strip ./target/release/nho



FROM debian:bookworm-slim AS release
WORKDIR /app
COPY --from=builder /app/target/release/nho .

EXPOSE 6379 
CMD ["./nho"]
