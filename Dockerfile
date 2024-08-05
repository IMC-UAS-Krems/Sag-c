FROM --platform=linux/amd64 rust:1-slim AS build

WORKDIR /app
COPY . .
COPY Cargo.lock Cargo.lock

RUN apt-get update -y && apt-get install -y pkg-config libssl-dev

RUN CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
cargo build --locked --target x86_64-unknown-linux-gnu	--release -p sagc

FROM gcr.io/distroless/cc-debian12:latest

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-gnu/release/sagc .
EXPOSE 8080
CMD ["/app/sagc"]
