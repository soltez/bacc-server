# bacc-server

A Baccarat scoreboard REST API server powered by [bacc-rs](https://github.com/soltez/bacc-rs).

## Overview

Maintains an internal `BaccaratShoe` and `BaccaratScoreboard`. A background task plays one
round every 30 seconds. When the shoe is exhausted, a fresh shoe is created and the
scoreboard resets automatically.

## Endpoints

### `GET /scoreboard`

Returns the current state of all five scoreboards as a JSON object.

**Response**

```json
{
  "bead_plate": "<hex string>",
  "big_road": "<hex string>",
  "derived_roads": ["<hex string>", "<hex string>", "<hex string>"]
}
```

All values are hex-encoded `BigUint` shift-registers as defined by `bacc-rs`:

- `bead_plate` - shift-register of bead bytes, newest at bits 0-7
- `big_road` - variable-width column shift-register, newest column at the low end
- `derived_roads` - Big Eye Boy, Small Road, and Cockroach Pig (run-length encoded)

## Running

```sh
cargo run
```

Server listens on `0.0.0.0:3000` by default.

## Configuration

Constants in `src/main.rs`:

| Constant              | Default | Description                        |
|-----------------------|---------|------------------------------------|
| `NUM_DECKS`           | `8`     | Number of decks in the shoe        |
| `PASSES`              | `1`     | Number of shuffle passes           |
| `PENETRATION`         | `0.75`  | Fraction of shoe dealt before cut  |
| `ROUND_INTERVAL_SECS` | `30`    | Seconds between rounds             |
