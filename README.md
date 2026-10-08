# Charm Adventure in Tomato Land

Two-player co-op platformer for the Steam Deck, written in Rust on Bevy.

Status: milestone 1, movement slice. Simon and Charm as cutout rigs (Tab swaps), one dark cave level, three tomato types.

## Download

Every push to `main` builds and publishes a release. Take `charm-adventure-steamdeck.tar.gz` from the latest release, then on the Deck in Desktop Mode:

```
tar xzf charm-adventure-steamdeck.tar.gz
cd charm-adventure && ./charm_adventure --fullscreen
```

## Controls

| Action | Deck | Keyboard |
|---|---|---|
| Move | Left stick / D-pad | Arrows / WASD |
| Jump | A | Space / Z |
| Attack | X | X / J |
| Dash (hold to sprint) | RT | C / K / Left Shift |
| Restart, fullscreen, quit | | R, F11, Esc |

## Editing

- `assets/config/tuning.ron`: player and tomato attributes. Saving applies within half a second.
- `assets/levels/rootway.ron`: platforms, spawns, tomato placement. Saving restarts the room.

## Build from source

```
sudo apt install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
cargo test --release
cargo run --release
```

## Layout

- `src/sim.rs`: deterministic gameplay simulation, no engine types.
- `src/main.rs`: window, input, rendering, hot reload.
