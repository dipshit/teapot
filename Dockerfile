FROM rust:1.36.0-slim

WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
WORKDIR /app
COPY --from=0 /app/target/x86_64-unknown-linux-musl/release/teapot /app/teapot
EXPOSE 8080
CMD ["/app/teapot"]
