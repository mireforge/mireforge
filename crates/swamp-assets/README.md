# swamp-assets 🐊

`swamp-assets` is a simple and efficient asset management system built on top of the `sparse-slot`. It is
designed to store and manage assets with unique identifiers, providing functionality for insertion, removal,
and access to assets. The system supports both immutable and mutable access to assets and is suitable for
use in game engines or any other system that needs to manage assets efficiently.

## ✨ Features

- **Efficient Asset Storage**: Uses `sparse-slot` for efficient handling of assets by index and generation.
- **Asset Management**: Add, remove, and access assets using unique identifiers (`Id`).
- **Immutable and Mutable Access**: Retrieve assets immutably or mutably with `get` and `get_mut`.
- **Asset Iteration**: Iterate over all stored assets with the `iter` method, providing both the `Id` and the asset.
- **Empty Check and Length**: Check if the asset store is empty or get the current number of stored assets with `is_empty` and `len`.

## 📦 Installation

To use `swamp-assets`, add it to your `Cargo.toml`:

```toml
[dependencies]
swamp-assets = "0.0.9"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


## Contents credits

- [Kenny.nl](https://kenney.nl/assets/platformer-art-deluxe) "platform art deluxe"
