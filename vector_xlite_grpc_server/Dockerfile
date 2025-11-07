# Stage 1: Build
FROM rust:1.91 as builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Set work directory inside container
WORKDIR /app/grpc_proto

COPY ./grpc_proto .


WORKDIR /app/vxlite

# Copy Cargo files first (for caching dependencies)
COPY ./vector_xlite_grpc_server/Cargo.toml ./vector_xlite_grpc_server/Cargo.lock ./

# Create a dummy main.rs so dependencies can build
RUN mkdir src && echo "fn main(){}" > src/main.rs

# Build dependencies
RUN cargo build --release
RUN rm -f target/release/deps/my_rust_app*

COPY ./vector_xlite_grpc_server/. .
RUN cargo build --release

#Stage 2 : Run
FROM debian:bookworm-slim

WORKDIR /root/

COPY --from=builder  /app/vxlite/target/release/vector_xlite_grpc .

ENV GRPC_ADDR=0.0.0.0:50051

EXPOSE 50051

CMD [ "./vector_xlite_grpc" ]