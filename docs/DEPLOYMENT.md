# Deployment

## Local process

```bash
export NUOFIELD_BIND=127.0.0.1:3000
export NUOFIELD_DATA_DIR="$PWD/data"
cargo run --release -p nuofield-server
```

Configuration:

| Variable | Default | Meaning |
|---|---|---|
| `NUOFIELD_BIND` | `127.0.0.1:3000` | HTTP listen address |
| `NUOFIELD_DATA_DIR` | `./data` | Deployment-owned data directory |
| `RUST_LOG` | server and HTTP info | Structured log filter |

## Docker Compose

```bash
docker compose up --build -d
docker compose logs -f nuofield
curl --fail http://127.0.0.1:3000/readyz
```

The named volume `nuofield-data` contains `events.jsonl`. Back it up while the
single writer is stopped:

```bash
docker compose stop nuofield
docker run --rm \
  -v nuofield-data:/data:ro \
  -v "$PWD/backup":/backup \
  alpine tar -C /data -czf /backup/nuofield-data.tgz .
docker compose start nuofield
```

## Container registry

Successful pushes to `main` publish:

```text
ghcr.io/homodeus/nuofield:main
ghcr.io/homodeus/nuofield:sha-<commit>
```

Version tags such as `v0.1.0` also publish semantic-version image tags.

## Network boundary

The current milestone has no cryptographic request authentication. The
container binds on port 3000 for local evaluation, but operators must keep it
behind a trusted network boundary. Do not publish the port to the internet.
