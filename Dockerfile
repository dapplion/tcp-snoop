FROM rust:alpine as cargo-build

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

# ------------------------------------------------------------------------------
# Package Stage
# ------------------------------------------------------------------------------

FROM alpine

# create user to limit access in container
RUN groupadd -g 1001 tcp_snoop && useradd -r -u 1001 -g tcp_snoop tcp_snoop

WORKDIR /home/tcp_snoop/bin/

COPY --from=cargo-build /usr/src/target/release/tcp_snoop .

RUN chown -R tcp_snoop:tcp_snoop /home/tcp_snoop/

USER tcp_snoop

ENTRYPOINT [""]