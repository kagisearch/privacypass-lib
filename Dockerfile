FROM rust:1.84 AS builder

# Install dependencies

RUN bash -c "curl -fsS https://rustwasm.github.io/wasm-pack/installer/init.sh | sh"

# Copy source

COPY src /src

# Build library

WORKDIR /src
RUN bash build.sh

# Copy library to host

CMD [ "cp", "-r", "/src/wasm/pkg", "/build/wasm" ]
