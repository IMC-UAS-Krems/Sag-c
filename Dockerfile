FROM rust:1-slim AS build

WORKDIR /app
COPY . .
COPY Cargo.lock Cargo.lock

RUN update-ca-certificates
RUN apt-get update -y && apt-get install -y pkg-config libssl-dev

RUN cargo build --locked --release -p sagc
RUN cargo build --locked --release -p sagc

FROM --platform=linux/arm64 gcr.io/distroless/cc-debian12:latest

WORKDIR /app
COPY --from=build /app/target/release/sagc .
COPY --from=build /app/target/release/sagc .
EXPOSE 8080
CMD ["/app/sagc"]