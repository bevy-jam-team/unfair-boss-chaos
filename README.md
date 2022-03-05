# Unfair Boss Chaos

> A battle against a boss you think you can defeat..
> Make with Rust in a week for the Bevy Jam #1

![+NwAp7](https://user-images.githubusercontent.com/4984415/156893451-9c77b22a-7d78-4723-af98-c464bc4e6a2e.png)

Play the game here: https://myisaak.itch.io/unfair-boss-chaos

The code is quite messy, due to having only a week. So apologies beforehand.

## Features

- Pathbinding system
- Shooting system
- integrated Physics with Rapier
- WASM Build

## Getting it to run locally on your browser

```
rustup target add wasm32-unknown-unknown
cargo install wasm-server-runner
cargo run --release
```

Point browser to [http://127.0.0.1:1334](http://127.0.0.1:1334)
