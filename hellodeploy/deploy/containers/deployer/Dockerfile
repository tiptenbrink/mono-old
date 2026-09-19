# Slim Debian container with Rust toolchain (including the cargo package manager) installed
FROM rust:1-slim-bookworm

# to easily download binaries, we need curl. we don't want our container to be too large so we delete the cache afterwards
RUN apt-get update \
	&& apt-get install -y curl \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /var/cache/apt/*

# quickinstall is easy to compile
RUN cargo install cargo-quickinstall
# we then get cargo-binstall as a binary using quickinstall
RUN cargo quickinstall cargo-binstall
# we can now easily get these as binaries also
RUN cargo binstall bws -y
RUN cargo binstall nu -y

WORKDIR /dployer

COPY entrypoint.nu .

ENTRYPOINT ["./entrypoint.nu"]