FROM rust:slim AS builder
WORKDIR /
ENV TARGET=x86_64-unknown-linux-musl
RUN rustup target add $TARGET

RUN apt-get update -y && \
    apt-get install -y git && \
    apt-get install -y musl-tools

RUN git clone https://github.com/ChangeCaps/dew.git
WORKDIR /dew
RUN git checkout $COMMIT
RUN cargo build --release --target $TARGET --package back

FROM scratch

ENV SSL_CERT=/run/secrets/cert.pem
ENV SSL_KEY=/run/secrets/key.pem
ENV DATA=/db-data/data.ron

COPY --from=builder /dew/target/x86_64-unknown-linux-musl/release/back /bin/server
CMD ["/bin/server"]
