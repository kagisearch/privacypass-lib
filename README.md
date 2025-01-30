# Kagi Privacy Pass Core Library

This repository contains the source code of the core library implementing the Privacy Pass API used by Kagi.
This repository is not meant to be used as stand-alone, but rather as a submodule for other projects.

## Build using Docker

To build this library, install Docker and run `bash build.sh`. If using Podman, run `DOCKER=podman bash build.sh` instead.
The output library will be found in `/build`.

## Build in host machine

To build this project directly, you need Rust and wasm-pack.
Run
```bash
cd src
bash build.sh
```
The output library will be found in `/src/wasm/pkg`.
