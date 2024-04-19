FROM rust:1-buster AS build

WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN USER=root apt update && apt install -y musl-tools

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release -p sagc

FROM alpine:latest

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/sagc .
EXPOSE 8080
CMD ["/app/sagc"]
