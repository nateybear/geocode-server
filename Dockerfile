FROM rust:1.78.0-alpine3.20

COPY ./ ./

RUN apk add musl-dev

ENV SQLX_OFFLINE=true

ENV RUST_LOG=info

RUN cargo build --release

RUN rm .env

CMD ["./target/release/geocode-server"]
