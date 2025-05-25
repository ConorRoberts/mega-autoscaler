ARG TARGET=aarch64-unknown-linux-gnu
ARG BIN=main

FROM rust:1.87-bookworm AS builder

ARG TARGET
ARG BIN

WORKDIR /app

COPY . .


# Install required system packages
RUN apt-get update && apt-get install -y \
  build-essential \
  pkg-config \
  cmake \
  clang \
  git \
  curl \
  ca-certificates \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

RUN rustup target add ${TARGET}
RUN cargo build --release --bin ${BIN} --target ${TARGET}

FROM gcr.io/distroless/cc-debian12

WORKDIR /app

ARG TARGET
ARG BIN

COPY --from=builder /app/target/${TARGET}/release/${BIN} .

CMD ["./main", "-d", "nginx:latest"]
