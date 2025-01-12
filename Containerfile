FROM docker.io/library/rust:1.82 AS build

WORKDIR /app
COPY . .

RUN cargo build --profile release-lto --features compression

FROM registry.access.redhat.com/ubi9/ubi-micro

LABEL org.opencontainers.image.source=https://github.com/xoogware/crawlspace
LABEL org.opencontainers.image.description="a tiny limbo server"
LABEL org.opencontainers.image.licenses=AGPL-3.0-or-later

WORKDIR /app
COPY --from=build /app/target/release-lto/crawlspace .

CMD ["/app/crawlspace"]
