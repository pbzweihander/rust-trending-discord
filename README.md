# rust-trending-discord

[![docker automated build](https://img.shields.io/docker/build/pbzweihander/rust-trending-discord.svg)](https://hub.docker.com/r/pbzweihander/rust-trending-discord/)
[![license: MIT](https://badgen.net/badge/license/MIT/green)](LICENSE)

<img src="logo.svg" alt="Thinking With Rust" width="300px">

A discord bot to post [trending rust repositories](https://github.com/trending/rust), converted from [twitter bot](https://github.com/pbzweihander/rust-trending).

## Usage

### Requirements

- Redis

### Local

```bash
cargo build --release
cargo install
rust-trending-discord config.toml
```

### Docker

```bash
docker run -p 6379:6379 --rm -d redis
docker run --rm -v $PWD/config.toml:/app/config.toml -d pbzweihander/rust-trending-discord:latest
```

### Docker Compose

```bash
cp config.toml /srv/rust-trending-discord/config.toml
docker-compose up -d
```

## License

This project is licensed under the terms of [MIT license](LICENSE).
