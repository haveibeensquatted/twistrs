FROM rust:1.46 as builder
WORKDIR /usr/src/twistrs-grpc

COPY proto/ proto/
COPY Cargo.toml Cargo.toml
COPY build.rs build.rs
COPY src/server.rs src/server.rs
COPY src/domain_enumeration.rs src/domain_enumeration.rs

RUN rustup component add rustfmt
RUN cargo build --bin server

WORKDIR /usr/src/twistrs-grpc/target/debug

CMD ["./server"]