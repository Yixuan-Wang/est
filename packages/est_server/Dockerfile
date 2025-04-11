ARG APP_NAME=est_server

FROM rust:alpine AS builder

WORKDIR /app
COPY . /app

RUN cargo build --locked --release

FROM gcr.io/distroless/cc-debian12 as runtime

WORKDIR /app
COPY --from=builder /app/target/release/${APP_NAME} /app/${APP_NAME}

EXPOSE 3000
CMD ["./${APP_NAME}"]
