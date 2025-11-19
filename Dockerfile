FROM rust AS builder

WORKDIR /opt/kjv

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release

FROM debian

COPY --from=builder /opt/kjv/target/release/kjv /usr/local/bin/kjv

CMD /usr/local/bin/kjv
