# Installation

> **Note:** An installation of the Rust programming language is needed to compile Antimony.

## Cargo

The fastest way to get up and running is to install the [latest published version](https://crates.io/crates/antimony-lang) via cargo:

```sh
cargo install antimony-lang
```

## Git

To get the current development version, you can clone the Git [repository](https://github.com/garritfra/antimony) and run the following command:

```sh
cargo install --path .
```

## Docker

Antimony provides a [Docker image](https://hub.docker.com/r/garritfra/antimony). It currently only supports the x64 architecture. Please reach out if you need a ARM variant (needed for Raspberry Pi). If you don't want to wait, you can build the image yourself by running this command in the root of the project:

```
docker build . -t antimony
```
