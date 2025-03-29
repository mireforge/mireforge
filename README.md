# MireForge

Welcome to the **Mireforge** 2D Game Engine!

Mireforge is a lightweight 2D game engine designed for simplicity and performance, offering features for 2D rendering and audio support. It targets multiple platforms, including Windows, MacOS, Linux, and WebAssembly (WASM).

It is designed for a smaller games, and is very opinionated.

## ✨ Features

- **2D Rendering**: Efficient rendering capabilities for sprites, tilemaps, and text
- **Audio Support**: Built-in stereo audio playback for sound effects and music
- **Cross-Platform**: Runs on Windows, MacOS, Linux, and Web (WASM)
- **Asset Management**: Robust asset loading and management system
- **Easy to Use**: Intuitive API to help you get started quickly

## 📦 Installation

Add mireforge to your project’s Cargo.toml:

```toml
[dependencies]
mireforge = "0.0.11"
```

## Run the examples

- Go into the examples crate:

```bash
cd crates/examples
```

- follow the instructions in:**[`examples`](crates/examples/README.md)**.

## Crates Overview

| Crate          | Description                                                          |
| -------------- | -------------------------------------------------------------------- |
| `swamp`        | Main engine crate that ties everything together                      |
| `game`         | Core game types and traits for building 2D games (Application trait) |
| `render`       | Render types and non render api specific code                        |
| `render-wgpu`  | WebGPU implementation of the render traits                           |
| `wgpu`         | Low-level WebGPU utility functions                                   |
| `wgpu-sprites` | Sprite rendering utilities for WebGPU                                |
| `game-assets`  | Asset loading and management system                                  |
| `game-audio`   | Audio playback                                                       |
| `font`         | Font info loading                                                    |
| `material`     | Material and texture management                                      |
| `boot-game`    | Game bootstrapping and initialization                                |
| `examples`     | Example games and demos                                              |

## Recommended Resolutions

### Common Desktop Resolutions

- 1920 x 1080
- 2560 x 1440
- 3840 x 2160

### Steam Deck Resolution

The Steam Deck has a unique resolution of **1280 × 800**, which can be approximated to a **720p**-like resolution.

### Base Resolutions for Pixel-Perfect Retro Feel

For these resolutions, **720p** serves as a common denominator. To achieve a retro, pixel-perfect aesthetic, we recommend the following base resolutions:

- **640 x 360**
- **320 x 180** (for a truly old-school feel)

If your target resolution is 1080p or higher, it can work with an intermediate base resolution such as:

- **384 × 216**

This approach allows for flexibility and maintains visual clarity across various display sizes.

## About Contributions

This is an open source project with a single copyright holder.
While the code is publicly available under [LICENSE](LICENSE), I am not accepting external contributions at this time.

You are welcome to:

- Use the code according to the license terms
- Fork the project for your own use
- Report issues
- Provide feedback
- Share the project

If you have suggestions or find bugs, please feel free to open an issue for discussion. While I cannot accept pull requests, I value your feedback and engagement with the project.

Thank you for your understanding and interest in the project! 🙏

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

_Copyright (c) 2024 Peter Bjorklund. All rights reserved._
