# bacc-server

A Baccarat REST API server powered by [bacc-rs](https://github.com/soltez/bacc-rs).

## Overview

Maintains an internal `BaccaratShoe` and `BaccaratScoreboard`. Round advancement is
client-driven via `POST /round/next`. When the shoe is exhausted, a fresh shoe is
created and the scoreboard resets automatically.

## Endpoints

### `POST /round/next`

Advances the shoe by one round, updates all scoreboards, and returns the round.

**Response**

```json
{
  "encoded":          123456,
  "is_forced_third":  false,
  "cut_card_index":   null,
  "player_cards":     [268471337, 134253349],
  "banker_cards":     [67115551, 268454953]
}
```

- `encoded` - packed `u32` round outcome as defined by `bacc-rs`
- `is_forced_third` - true if the banker drew a forced third card
- `cut_card_index` - `0` means last round; `1-5` means one more round follows; `null` means cut card not seen
- `player_cards` / `banker_cards` - Cactus Kev `u32` card integers

### `GET /round`

Returns the round from the most recent `POST /round/next` call. Returns `204 No Content`
if no round has been played yet. Response shape is identical to `POST /round/next`.

### `GET /scoreboard`

Returns the scoreboard state reflecting the most recent round.

**Response**

```json
{
  "bead_plate":    "<hex string>",
  "big_road":      "<hex string>",
  "derived_roads": ["<hex string>", "<hex string>", "<hex string>"]
}
```

All values are hex-encoded `BigUint` shift-registers as defined by `bacc-rs`:

- `bead_plate` - shift-register of bead bytes, newest at bits 0-7
- `big_road` - variable-width column shift-register, newest column at the low end
- `derived_roads` - Big Eye Boy, Small Road, Cockroach Pig (run-length encoded)

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
| `PASSES`      | `1`     | Number of shuffle passes          |
| `PENETRATION` | `0.75`  | Fraction of shoe dealt before cut |
