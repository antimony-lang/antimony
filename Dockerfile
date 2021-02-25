FROM rust:1.50.0 AS build
WORKDIR /usr/src

# Download the target for static linking.
RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
WORKDIR /usr/src/antimony
COPY Cargo.toml Cargo.lock ./

# Copy the source and build the application.
COPY src ./src
COPY lib ./lib
COPY builtin ./builtin
RUN cargo install --target x86_64-unknown-linux-musl --path .

RUN cargo build --release

# Copy the statically-linked binary into a scratch container.
FROM alpine:3.13

LABEL maintainer="Garrit Franke <garrit@slashdev.space>"

COPY --from=build /usr/local/cargo/bin/antimony /bin

RUN antimony --version

ENTRYPOINT ["antimony"]
