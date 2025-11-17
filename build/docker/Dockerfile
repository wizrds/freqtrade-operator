ARG RUST_VERSION=1.90.0
FROM rust:${RUST_VERSION}-slim-bookworm AS build

ARG RUST_VERSION=1.90.0

# Install necessary dependencies for building
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set the current working directory
WORKDIR /workspace

# Copy the source files
COPY . .

# Build the application with cargo
RUN cargo build --release

# Runtime Stage
FROM ubuntu:22.04 AS runtime

WORKDIR /app

COPY --from=build /workspace/target/release/freqtrade-operator /app/freqtrade-operator

ENTRYPOINT ["/app/freqtrade-operator"]
CMD ["controller"]