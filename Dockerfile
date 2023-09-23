# FROM rust:1.72-slim-bookworm as builder
FROM rust:1.72-alpine3.18 as builder

RUN apk add --no-cache musl-dev

WORKDIR /mordor

COPY . .
RUN cargo build --release


FROM alpine:3.18
# FROM debian:bookworm-slim
WORKDIR /mordor/data

COPY --from=builder /mordor/target/release/mordor /mordor/mordor
COPY config.sample.toml /mordor/data/config.toml

EXPOSE 8080

CMD ["/mordor/mordor", "-c", "/mordor/data/config.toml"]