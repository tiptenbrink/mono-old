FROM rust:1-slim-bookworm

RUN cargo install bws
RUN cargo install nu

WORKDIR /dployer

COPY entrypoint.nu .

ENTRYPOINT ["./entrypoint.nu"]