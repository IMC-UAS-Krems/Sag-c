FROM rust:1-slim-buster AS build

WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y --no-install-recommends clang llvm perl musl-tools
RUN update-ca-certificates
ENV CC_x86_64_unknown_linux_musl=clang
ENV RUST_BACKTRACE=full

# for Cargo
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release -p sagc

FROM alpine:latest

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/sagc .
EXPOSE 8080
CMD ["/app/sagc"]
