# Kagi Privacy Pass Core Library

This repository contains the source code of the core library implementing the Privacy Pass API used by [Kagi](https://blog.kagi.com/kagi-privacy-pass).
This repository is not meant to be used as stand-alone, but rather as a submodule for other projects.

## Building using Docker

To build this library, install Docker and run
```bash
bash build.sh
```
If using Podman, run
```bash
DOCKER=podman bash build.sh
```
The output library will be found in `/build`.

## Building on host machine

### Installing the build dependencies

To build this project directly on your host machine, you need [rust](https://www.rust-lang.org/) and [wasm-pack](https://rustwasm.github.io/wasm-pack/).

You can obtain Rust by using [rustup](https://rustup.rs/), and wasm-pack by using its [installer](https://rustwasm.github.io/wasm-pack/installer/).

### Building the library

Once the above dependencies were obtained, run
```bash
cd src
bash build.sh
```
The output library will be found in `/src/wasm/pkg`.
