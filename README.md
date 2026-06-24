# bacc-server

A Baccarat REST API server powered by [bacc-rs](https://github.com/soltez/bacc-rs) and
[bacc-core-rs](https://github.com/soltez/bacc-core-rs).

## Overview

Maintains an internal `BaccShoe` and `BaccScoreboard`. Round advancement is client-driven
via `POST /round/next`. When the shoe is exhausted, a fresh shoe is created and the
scoreboard resets automatically.

## Endpoints

### `POST /round/next`

Advances the shoe by one round, updates the scoreboard, and returns the encoded round.

**Response**

```json
{ "encoded_hex": "00a1b2c3d4e5f601" }
```

- `encoded_hex` - `BaccRound::encode()` hex string; clients decode via `bacc-core-rs`

### `GET /round`

Returns the encoded round from the most recent `POST /round/next` call. Returns
`204 No Content` if no round has been played yet. Response shape is identical to
`POST /round/next`.

### `GET /scoreboard`

Returns the encoded scoreboard state reflecting the most recent round.

**Response**

```json
{ "encoded_hex": "a1b2c3..." }
```

- `encoded_hex` - `BaccScoreboard::encode()` hex string; clients reconstruct all five
  roads via `decode()` and `simulate_*` from `bacc-core-rs`

## Running

```sh
cargo run
```

Server listens on `0.0.0.0:3000` by default.

## Configuration

Constants in `src/main.rs`:

| Constant      | Default | Description                       |
|---------------|---------|-----------------------------------|
| `NUM_DECKS`   | `8`     | Number of decks in the shoe       |
| `PASSES`      | `3`     | Number of shuffle passes          |
| `PENETRATION` | `0.965` | Fraction of shoe dealt before cut |
