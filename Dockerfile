FROM rust as builder

WORKDIR /opt/kjv

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release

FROM alpine

COPY --from=builder /opt/kjv/target/release/kjv /usr/local/bin/kjv

CMD ["kjv"]
