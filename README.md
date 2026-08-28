# Noro Launcher

Open-source cross-platform Minecraft launcher built with Rust and GPUI.

## Architecture
- **`crates/noro_launcher`**: Bootstrapper binary (handles self-updates and starts the core).
- **`crates/noro_core`**: Single-instance launcher daemon and bridge manager.
- **`crates/frontend`**: Native desktop GUI built on top of Zed's GPUI framework.
- **`crates/backend`**: Launcher business logic (auth, file sync, Java runtime, Minecraft runner).
- **`crates/bridge`**: Strongly-typed thread-safe IPC bridge between frontend and backend.
- **`crates/mod_link`**: Mod management and game link abstractions.

## Development

Prerequisites:
- Rust (stable toolchain, see `rust-toolchain.toml`)

Build and run:
```bash
cargo run -p noro_core
```
