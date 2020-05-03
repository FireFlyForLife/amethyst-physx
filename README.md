# amethyst-physx
This is a showcase and test suite of the [physx-rs](https://github.com/EmbarkStudios/physx-rs) library.
It uses my [own fork](https://github.com/FireFlyForLife/physx-rs) where I add new features/bindings to the library which I will feed back to the main repository.

## Current Features
- Debug lines

## Planned features
- Characters etc

## How to run
Firstly clone the repo with submodules:
`git clone --recurse-submodules -j8 https://github.com/FireFlyForLife/amethyst-physx.git`
(Or alternatively, initialize submodules after the fact)
`git clone https://github.com/FireFlyForLife/amethyst-physx.git`
`git submodule update --init --recursive -j8`

To run the game, run the following command, which defaults to the `vulkan` graphics backend:

```bash
cargo run
```

Windows and Linux users may explicitly choose `"vulkan"` with the following command:

```bash
cargo run --no-default-features --features "vulkan"
```

Mac OS X users may explicitly choose `"metal"` with the following command:

```bash
cargo run --no-default-features --features "metal"
```
