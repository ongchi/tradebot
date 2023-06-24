FROM rust:latest

RUN apt-get -q update && apt-get -yq install \
  libssl-dev && \
  apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-gnu"]
