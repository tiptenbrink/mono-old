FROM rust:1-slim-bookworm

RUN cargo install bws -y
RUN cargo install nu -y

WORKDIR /dployer

COPY entrypoint.nu .

ENTRYPOINT ["./entrypoint.nu"]