FROM rust:slim AS builder
WORKDIR /
ENV TARGET=x86_64-unknown-linux-musl
RUN rustup target add $TARGET

RUN apt-get update -y && \
    apt-get install -y git && \
    git clone https://github.com/ChangeCaps/dew && \
    cd dew && \
    cargo build --release --locked --target $TARGET --bin back

FROM scratch
COPY --from=builder /dew/target/x86_64-unknown-linux-musl/release/back /bin/server
CMD ["/bin/server"]
