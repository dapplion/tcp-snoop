FROM rust:latest as cargo-build

WORKDIR /usr/src/

COPY . .

# RUN rustup target add x86_64-unknown-linux-musl
# RUN cargo build --release --target x86_64-unknown-linux-musl
RUN cargo build --release

# ------------------------------------------------------------------------------
# Package Stage
# ------------------------------------------------------------------------------

FROM ubuntu:focal

LABEL org.opencontainers.image.source=https://github.com/dapplion/tcp-snooper

# create user to limit access in container
RUN groupadd -g 1001 tcp-snooper && useradd -r -u 1001 -g tcp-snooper tcp-snooper

WORKDIR /home/tcp-snooper/bin/

COPY --from=cargo-build /usr/src/target/release/tcp-snooper .

RUN chown -R tcp-snooper:tcp-snooper /home/tcp-snooper/

USER tcp-snooper

ENTRYPOINT [""]
