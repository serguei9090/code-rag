# Dockerfile test asset

FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=builder /app/target/release/code-rag /usr/local/bin/
CMD ["code-rag", "search", "hello"]
