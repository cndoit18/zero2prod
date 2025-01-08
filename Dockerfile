FROM lukemathwalker/cargo-chef:latest-rust-1.82 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod

FROM rust:1.82 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration.yaml configuration.yaml
ENTRYPOINT ["./zero2prod"]
