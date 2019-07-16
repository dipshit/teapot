FROM rust:1.36.0-slim

WORKDIR /app

# For faster builds only refetch when Cargo.toml or Cargo.lock changes
# cargo fetch needs a main file before it can run, so we stub one
RUN mkdir /app/src
RUN echo 'fn main() {}' > src/main.rs
COPY Cargo.toml Cargo.lock ./
RUN rustup target add x86_64-unknown-linux-musl && cargo build --release
RUN rm src/main.rs

COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
WORKDIR /app
COPY --from=0 /app/target/x86_64-unknown-linux-musl/release/teapot /app/teapot
EXPOSE 8080
CMD ["/app/teapot"]
