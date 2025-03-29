# Mireforge examples 🐊

The Mireforge Examples are designed to help developers quickly get started with the Mireforge Render API
by providing ready-to-run examples.

## Usage

- List the available examples:

```bash
cargo run --bin
```

- Run one of the examples with:

```bash
cargo run --release --bin <example_name>
```

Replace <example_name> with the specific example you want to run.

- To see detailed logs of the application’s behavior, you can run with logging enabled:

```bash
RUST_LOG=debug,wgpu_core=warn,naga=warn,wgpu_hal=warn,winit=warn cargo run --example <example_name>
```

Replace <example_name> with the specific example you want to run.

## Asset Licenses

All assets used in the examples are sourced from platforms that provide open and free licenses.
Please refer to each asset’s respective page for specific licensing information.

- **Jerom**: Creator of the [32x32-fantasy-tileset](https://opengameart.org/content/32x32-fantasy-tileset) used in the
  `single_texture` example.
- **Atari Boy**: Creator of the [Old hero Pixel Art character](https://atari-boy.itch.io/oldherocharacter) used in the
  `animation` example.
- **VEXED**: Creator
  of [bountiful bits](https://v3x3d.itch.io/bountiful-bits) [(1)](https://opengameart.org/content/bountiful-bits-10x10-top-down-rpg-tiles).
- **Peter**: Creator of the menu font.
- **Pixel Frog**: Creator of [Pixel Adventure 2](https://pixelfrog-assets.itch.io/pixel-adventure-2)

- **qubodup** created sound [Whoosh](https://freesound.org/people/qubodup/sounds/60013/)
