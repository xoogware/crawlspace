FROM docker.io/library/rust:1.82 AS build

WORKDIR /app
COPY . .

RUN cargo build --profile release-lto --features compression

FROM registry.access.redhat.com/ubi9/ubi-micro
WORKDIR /app
COPY --from=build /app/target/release-lto/crawlspace .

CMD ["/app/crawlspace"]
