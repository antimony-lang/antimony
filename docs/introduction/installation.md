# Installation

> **Note:** An installation of the Rust programming language is needed to compile Sabre.

## Cargo

The fastest way to get up and running is to install the [latest published version](https://crates.io/crates/sabre-lang) via cargo:

```sh
cargo install sabre-lang
```

## Git

To get the current development version, you can clone the Git [repository](https://github.com/garritfra/sabre) and run the following command:

```sh
cargo install --path .
```

## Docker

Sabre provides a [Docker image](https://hub.docker.com/r/garritfra/sabre). It currently only supports the x64 architecture. Please reach out if you need a ARM variant (needed for Raspberry Pi). If you don't want to wait, you can build the image yourself by running this command in the root of the project:

```
docker build . -t sabre
```
