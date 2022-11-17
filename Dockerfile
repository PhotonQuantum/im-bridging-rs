FROM rustlang/rust:nightly-slim AS builder

WORKDIR /work

RUN apt update && apt install -y libssl-dev pkg-config

COPY ./src ./src

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /work

RUN apt update && apt install -y libssl-dev ca-certificates

COPY --from=builder ./work/target/release/im-bridging-rs ./

ENV SESSION_FILE=/data/session.json

ENV DEVICE_FILE=/data/device.json

CMD ["./im-bridging-rs"]