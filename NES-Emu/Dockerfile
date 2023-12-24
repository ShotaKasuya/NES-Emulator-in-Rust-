FROM wjlroe/docker-ubuntu-rust:latest

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install \
    uuid-dev \
    libssl-dev \
    libsdl2-dev \
    libsdl2-gfx-dev \
    libsdl2-image-dev \
    libsdl2-mixer-dev \
    libsdl2-net-dev \
    libsdl2-ttf-dev \
    libfreetype6-dev \
    tk-dev \
    -qqy \
    --no-install-recommends \
    curl zip unzip tar \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives

RUN rustup default nightly && rustup update
RUN cargo install cargo-vcpkg
# RUN cargo vcpkg build

ENV LAST_RUST_IMAGE_UPDATE=2018-10-13

RUN rustc --version