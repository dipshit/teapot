FROM rust:1.36.0

WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:1.36.0
WORKDIR /app
COPY --from=0 /app/target/release/teapot /app/teapot
EXPOSE 8080
CMD ["/app/teapot"]
