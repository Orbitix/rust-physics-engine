# Rust Physics Engine

A simple ball physics solver written in Rust

## Versions

This project contains two versions:

### Version 2D (Bevy) - **Recommended**
- Uses Bevy 0.15.0 game engine
- Modern, safe, and actively maintained
- Build with: `cargo build --bin version_2d`
- Run with: `cargo run --bin version_2d`

### Version 3D (Macroquad) - **Not Recommended**
- Uses Macroquad for 3D rendering
- ⚠️ **Security Warning**: The macroquad dependency (v0.4.14) has known soundness issues and no patched version is available
- Only build if you understand the risks: `cargo build --bin version_3d --features version_3d`

## Building

By default, only the safe Bevy 2D version is built:
```bash
cargo build --bin version_2d
cargo run --bin version_2d
```

To build the 3D version (not recommended due to security vulnerabilities):
```bash
cargo build --bin version_3d --features version_3d
```

## Configuration

Edit `config.toml` to adjust simulation parameters like ball count, gravity, resistance, etc.
