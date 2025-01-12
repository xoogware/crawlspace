# crawlspace
a tiny limbo server

for now, this only supports a very tiny subset of the minecraft protocol, and will generally be kept up to date with the latest major changes as it's intended to be behind Velocity with ViaBackwards. 
it's possible that it'll work on older minecraft versions but honestly I am not intentionally trying to support that

(i don't know rust! this is a work in progress and will be rewritten many times to be More Correct! gimme a sec let me cook!)

> [!IMPORTANT]
> For now, Crawlspace is **extremely** limited in the functionality it provides.
> It is NOT intended to be a fully-featured Minecraft server; in fact, it's quite the opposite.
> Crawlspace does **not** implement a full entity tick loop or anything of the sort; instead, it is first and foremost meant to be the **bare minimum** required to keep a player connected.
>
> Crawlspace is intended to be optimized for maximum player count above all else, and only secondarily as a storytelling medium for XOOGWARE.
> This means that Crawlspace can currently **only load worlds that have been specially prepared for it**: currently, only the End is supported, and chunks are loaded **immediately** and sent to the player, so world size should be kept as small as possible (between chunks -10 and 10 on both axes, inclusive).

# Running
Download a precompiled binary, if available, or [build from source](#build-from-source).
You can also [use a container image](#use-a-container-image).

## Build from source
Clone the repo:
```bash
git clone https://github.com/xoogware/crawlspace
```
Ensure you have a compatible version of Rust installed. The toolchain we use can be found in [rust-toolchain.toml](https://github.com/xoogware/crawlspace/blob/master/rust-toolchain.toml).
If you use Rustup this will be installed automatically when you run any Cargo command.

Then, simply:
```bash
# build
cargo build
# ...or run
RUST_LOG=info cargo run path/to/your/world
```

Crawlspace also has profiles to enable stripping and LTO:
```bash
cargo build --profile=release-strip
cargo build --profile=release-lto
```

## Use a container image
Nightly builds of Crawlspace are pushed to GHCR, and tagged version releases will be as well.
Pull using either `nightly` or a commit short hash as the tag:
```bash
podman pull ghcr.io/xoogware/crawlspace:nightly
```
then run with the world mounted read-only:
```bash
podman run --rm --read-only -v=./tmp/DIM1:/world:ro,Z -e="LIMBO_WORLD=/world" -p=8006:25565 crawlspace
```

# Configuration
Crawlspace supports multiple modes of configuration. In order of priority, with first being the highest:

1. Command line flags (run `crawlspace --help` for more info)
2. Environment variables (see [Environment Variables](#environment-variables))
3. (TO BE IMPLEMENTED) Lua Scripting API (see [Lua Scripting](#lua-scripting))

## Environment Variables
Environment variables can be provided to configure basic Crawlspace functionality.
Please note that **environment variables will be overridden by command line flags if passed.**

- `LIMBO_ADDRESS`: The address to host the server on. Defaults to `[::]`.
- `LIMBO_PORT`: The port to host the server on. Defaults to `25565`.
- `LIMBO_MAX_PLAYERS`: the hard player limit. connections will be refused past this
- `LIMBO_WORLD`: the directory to load the map from. Should be DIM1, or the equivalently named folder.
- `LIMBO_SPAWN_X`, `LIMBO_SPAWN_Y`, and `LIMBO_SPAWN_Z`: The coordinates to spawn the player at. Defaults to (0, 100, 0).
- `LIMBO_BORDER_RADIUS`: The radius of the world border, in blocks, centered on the spawnpoint. Defaults to 10 chunks.

## Lua Scripting
**To be implemented.** 
Crawlspace will feature a Lua scripting API to configure things such as map loading, spawning, etc. along with reactions to basic player events such as movement.
