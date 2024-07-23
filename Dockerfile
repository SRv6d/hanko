FROM rust:1-slim-bookworm AS build

ARG target

RUN apt-get update && \
    apt-get install -y gcc-aarch64-linux-gnu
RUN rustup target add ${target}

WORKDIR /usr/src/hanko
COPY . .

RUN cargo build --release --locked --target ${target}


FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="hanko"
LABEL org.opencontainers.image.authors="Marvin Vogt <m@rvinvogt.com>"

COPY --from=build /usr/src/hanko/target/release/hanko /app/hanko

ENTRYPOINT ["/app/hanko"]
