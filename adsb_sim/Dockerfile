# syntax=docker/dockerfile:1

# Stage 1: build static adsb_sim binary (musl, no shared libs)
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build
COPY rcal ./rcal

WORKDIR /build/adsb_sim
COPY adsb_sim/Cargo.toml adsb_sim/Cargo.lock adsb_sim/CALConfig.toml ./
COPY adsb_sim/.cargo ./.cargo
COPY adsb_sim/src ./src

RUN cargo build --release --locked --target x86_64-unknown-linux-musl -p adsb_sim

# Stage 2: build final image with some default data.
FROM scratch

COPY adsb_sim/data/ /data
COPY adsb_sim/CALConfig.toml /CALConfig.toml
COPY --from=builder /build/adsb_sim/target/x86_64-unknown-linux-musl/release/adsb_sim /adsb_sim

ENV RCAL_CALCONFIG_PATH=/CALConfig.toml

CMD ["/adsb_sim"]
