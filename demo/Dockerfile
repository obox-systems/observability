FROM lukemathwalker/cargo-chef:latest-rust-1.81.0 as chef
WORKDIR /app
RUN apt update && apt-get upgrade -y && apt install libssl-dev -y

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
# Build our project

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt update \
    && apt-get update -y \
    && apt-get install libssl-dev -y \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/demo demo
COPY --from=builder /app/docker_entry.sh docker_entry.sh

RUN chmod +x ./docker_entry.sh

EXPOSE 5000 9000

ENTRYPOINT ["./docker_entry.sh"]