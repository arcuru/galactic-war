FROM rust:bookworm AS builder
WORKDIR /usr/src/galactic-war
COPY . .
RUN cargo build --release --bin galactic-war

FROM debian:bookworm-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/galactic-war/target/release/galactic-war /usr/local/bin/galactic-war
CMD ["galactic-war"]
