# ============================
# Stage 1: Build
# ============================
FROM rust:1.91 as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Copy and compile protobuf definitions
WORKDIR /app/grpc_proto
COPY ./grpc_proto .

# Set up Rust project workspace
WORKDIR /app/vxlite

# Copy Cargo files first (for caching dependencies)
COPY ./vector_xlite_grpc_server/Cargo.toml ./vector_xlite_grpc_server/Cargo.lock ./

# Create a dummy main.rs so dependencies can build
RUN mkdir src && echo "fn main(){}" > src/main.rs

# Build dependencies
RUN cargo build --release
RUN rm -f target/release/deps/my_rust_app*

# Copy actual project source
COPY ./vector_xlite_grpc_server/. .

# Build final optimized binary
RUN cargo build --release --locked

# ============================
# Stage 2: Runtime
# ============================
FROM debian:bookworm-slim

# Install SQLite runtime dependency
RUN apt-get update && \
    apt-get install -y --no-install-recommends libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*


WORKDIR /root/

# Copy binary from builder stage
COPY --from=builder  /app/vxlite/target/release/vector_xlite_grpc .

# Configure environment variables
ENV GRPC_ADDR=0.0.0.0:50051

# Expose gRPC port
EXPOSE 50051

# Run the gRPC server
CMD ["./vector_xlite_grpc"]