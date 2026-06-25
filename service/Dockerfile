FROM rust:1.88 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
RUN cargo build --release

FROM debian:bookworm-slim AS runner
WORKDIR /app
COPY --from=builder /app/target/release/service ./service
ENV API_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["./service"]
