FROM rust:latest

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV SQLX_OFFLINE=true

COPY . .

RUN cargo install sqlx-cli --no-default-features --features postgres

RUN cargo build --release

EXPOSE 8080

CMD cargo run --release