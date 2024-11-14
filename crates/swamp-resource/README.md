# Swamp Resource 🐊

Welcome to the Swamp Resource crate!

This crate offers a flexible and type-safe storage system for managing
resources in your Rust applications. It allows you to dynamically store and retrieve resources using their type
identifiers, enabling seamless interaction with various types without sacrificing type safety.

Each resource can be thought of as a data singleton within the swamp-resource storage. By design, every resource 
type is stored as a single instance, ensuring you have a unique and easily accessible representation of each
resource throughout your application.

## ✨ Features

- **Dynamic Resource Storage**: Store resources of different types.
- **Type Safety**: Retrieve resources safely with downcasting to ensure type correctness.
- **Efficient**: Uses a `HashMap` for quick lookups and management of resources.
- **Simple API**: Easy-to-use methods for inserting, fetching, and managing resources.

## 📦 Installation

Add swamp-resource your project’s Cargo.toml:

```toml
[dependencies]
swamp-resource = "0.0.10"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
