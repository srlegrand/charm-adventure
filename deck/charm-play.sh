#!/bin/bash
# Charm Adventure on a Steam Deck: set up if needed, pull the latest, rebuild, run.
#
# Get it and start it (Desktop Mode, Konsole):
#   wget -O ~/charm-play.sh https://raw.githubusercontent.com/srlegrand/charm-adventure/main/deck/charm-play.sh && chmod +x ~/charm-play.sh && ~/charm-play.sh
# Then add /home/deck/charm-play.sh to Steam as a Non-Steam Game.
#
# SteamOS cannot compile the game itself, so the build runs in a small Ubuntu container (distrobox).
# The game then runs directly on SteamOS. Everything printed goes to ~/charm-play.log.
REPO="$HOME/charm-adventure"
LOG="$HOME/charm-play.log"
# In a terminal, show progress as well as logging it. From Steam there is nowhere to show it.
if [ -t 1 ]; then exec > >(tee "$LOG") 2>&1; else exec >"$LOG" 2>&1; fi
# Steam's own library settings break the container, so they are cleared for it.
box() { env -u LD_PRELOAD -u LD_LIBRARY_PATH distrobox "$@"; }

# First run only: the build container, its tools, Rust, and the clone.
if ! box list | grep -q charm-build; then
  box create -Y -n charm-build -i ubuntu:22.04
  box enter charm-build -- bash -lc 'sudo apt-get update && sudo apt-get install -y build-essential pkg-config git curl libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev'
fi
[ -x "$HOME/.cargo/bin/cargo" ] || box enter charm-build -- bash -lc 'curl -sSf https://sh.rustup.rs | sh -s -- -y'
[ -d "$REPO/.git" ] || git clone https://github.com/srlegrand/charm-adventure.git "$REPO"

cd "$REPO" || exit 1
git pull --ff-only || echo "PULL FAILED (offline?): running what is here"
# Lighter optimisation than the release builds, so a rebuild after a change takes about a minute, not many.
box enter charm-build -- bash -lc 'source ~/.cargo/env && cd ~/charm-adventure && CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_TARGET_DIR=target-deck cargo build --release --bin charm_adventure' \
  || echo "BUILD FAILED: running the last good build"
# The game looks for its assets folder beside the program.
mkdir -p target-deck/release && ln -sfn "$REPO/assets" target-deck/release/assets
exec ./target-deck/release/charm_adventure "$@"
