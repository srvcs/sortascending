# srvcs-sortascending

The ascending-sort service of the srvcs.cloud distributed standard library.

Its single concern: **sort a list of integers into ascending order.** It reads a
list of integers and returns the same values ordered from smallest to largest.

`srvcs-sortascending` is a **leaf**: it depends on no other service and makes no
network calls. All work is local.

```text
result = values sorted ascending
sortascending([3, 1, 2]) == [1, 2, 3]
```

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Sort `values` into ascending order |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"values": [3, 1, 2]}'
# {"values":[3,1,2],"result":[1,2,3]}

curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"values": [0, -5, 3, -5]}'
# {"values":[0,-5,3,-5],"result":[-5,-5,0,3]}
```

Responses:

- `200 {"values": [...], "result": [...]}` — evaluated. `result` is the elements
  of `values` sorted into ascending order.
- `422 {"error": "values must be integers"}` — some element of `values` is not a
  JSON integer.

The empty list yields the empty list. Duplicates are preserved and negatives are
ordered correctly.

## Dependencies

None. `srvcs-sortascending` is a leaf comparison service. Because it owns its own
validation, it rejects any non-integer element directly with `422` rather than
forwarding to a dependency.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See [`srvcs/platform`](https://github.com/srvcs/platform) for the shared
standard.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
