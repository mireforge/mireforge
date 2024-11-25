# Swamp 🐊

Welcome to the **Swamp** 2D Game Engine!

Swamp is a lightweight 2D game engine designed for simplicity and performance, offering features for 2D rendering and audio support. It targets multiple platforms, including Windows, MacOS, Linux, and WebAssembly (WASM).

It is designed for a smaller games, and is very opinionated. 


## ✨ Features

- **2D Rendering**: Efficient rendering capabilities for sprites, tilemaps, and text
- **Audio Support**: Built-in stereo audio playback for sound effects and music
- **Cross-Platform**: Runs on Windows, MacOS, Linux, and Web (WASM)
- **Asset Management**: Robust asset loading and management system
- **Easy to Use**: Intuitive API to help you get started quickly


## 📦 Installation

Add swamp to your project’s Cargo.toml:

```toml
[dependencies]
swamp = "0.0.11"
```

## Run the examples

- Go into the examples crate:

```bash
cd crates/examples
```
- follow the instructions in:**[`examples`](crates/examples/README.md)**.

## Crates Overview

| Crate | Description |
|-------|-------------|
| `swamp` | Main engine crate that ties everything together |
| `game` | Core game types and traits for building 2D games (Application trait) |
| `render` | Render types and non render api specific code |
| `render-wgpu` | WebGPU implementation of the render traits |
| `wgpu` | Low-level WebGPU utility functions |
| `wgpu-sprites` | Sprite rendering utilities for WebGPU |
| `game-assets` | Asset loading and management system |
| `game-audio` | Audio playback |
| `font` | Font info loading |
| `material` | Material and texture management |
| `boot-game` | Game bootstrapping and initialization |
| `examples` | Example games and demos |


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
