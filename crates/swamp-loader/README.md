# Swamp Loader 🐊

Welcome to **Swamp Loader**! This tiny but mighty library lets you load files as `Blob`s and send them wherever they're
needed, all in async style! Just think of it as a special delivery service for your asset files, 
from storage to wherever you need them in your app.

## ✨ Features

- Reads files asynchronously with any `AssetReader` you provide.
- Wraps the file path and content in a handy `Blob`.
- Sends the `Blob` through a channel so other parts of your app can use it.

## 📦 Installation

To use `swamp-loader`, add it to your `Cargo.toml`:

```toml
[dependencies]
swamp-loader = "0.0.9"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contents credits

- [Kenny.nl](https://kenney.nl/assets/platformer-art-deluxe) "platform art deluxe"