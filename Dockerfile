FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app

# Backend
#RUN rustup default nightly
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN USER=root cargo new --bin kuvakannu
WORKDIR /app/kuvakannu
COPY Cargo.lock Cargo.toml ./
COPY src src
COPY templates templates
COPY static static
RUN cargo build --release --bin kuvakannu

# Final container
FROM alpine AS runtime
RUN apk add --no-cache libgcc ffmpeg
WORKDIR /app
COPY --from=builder /app/kuvakannu/target/release/kuvakannu .
USER 1000
CMD [ "./kuvakannu" ]
