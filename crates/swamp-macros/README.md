# swamp-macros  🐊

`swamp-macros` is a procedural macro crate designed to simplify the implementation of common traits for swamp-resource type 
in Rust applications. It provides a convenient way to derive the `Resource` trait for your types.

## ✨ Features

- **Derive Macro**: Automatically implement the `Resource` trait for your structs.
- **Type Safety**: Ensures that only types marked as resources can be stored and managed by swamp-resource.
- **Ease of Use**: Reduce boilerplate code when implementing traits for multiple types.

## 📦 Installation

To use `swamp-macros`, add it to your `Cargo.toml`:

```toml
[dependencies]
swamp-macros = "0.0.10-dev3"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
