FROM rust:latest AS builder

WORKDIR /app
COPY . /app

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 as runtime

WORKDIR /app
COPY --from=builder /app/target/release/est_server /app/est_server

EXPOSE 3000
CMD ["./est_server"]
