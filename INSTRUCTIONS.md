# noro-launcher — Project Instructions

This is the single instruction file for `noro-launcher`.
Shared ideology, design tokens, and universal rules: **[../INSTRUCTIONS.md](../INSTRUCTIONS.md)**.
Short agent cheatsheet: **[./AGENTS.md](./AGENTS.md)** / **[./CLAUDE.md](./CLAUDE.md)**.

---

## 1. Overview & Architecture

**noro-launcher** is a native cross-platform desktop Minecraft launcher for a private server network.
It is a **public repository licensed under AGPL-3.0**.

### 1.1 Two Binaries, and Why

| Binary | Crate | Role |
|---|---|---|
| `noro-launcher` | `crates/noro_launcher` | Bootstrapper — the file a player installs |
| `noro-launcher-core` | `crates/noro_core` | The launcher itself, fetched, verified, and started by bootstrapper |

- **Reputation protection:** The bootstrapper **never updates itself** — that is how it accumulates Microsoft SmartScreen reputation on Windows.
- Everything that changes lives in `noro-launcher-core`.
- A change to the bootstrapper reaches existing installations only by reinstalling.

### 1.2 Crates Layout

```
crates/
  frontend/          # GPUI desktop interface (pages, components, theme.rs)
  backend/           # Launcher logic (auth, sync, JVM game runner, updater)
  bridge/            # Typed IPC protocol between UI thread and backend (mpsc)
  mod_link/          # Mod links and helper logic
  noro_core/         # Core binary (starts backend and frontend)
  noro_launcher/     # Bootstrapper binary (checks and launches core)
```

Shared crates (`schema`, `i18n`, `mc_mod_utils`) are consumed from the `noro-shared` repository over git.

---

## 2. Public AGPL-3.0 Hygiene

Because this repository is open-source and public:
- **Comments in English only.** This is the repo read by external developers.
- **No private infrastructure data:** No real personal domains, tokens, private paths, internal server IPs, or secrets in code, comments, or test fixtures. Use `example.com` as placeholder.
- **Strict i18n:** All player-visible strings must go through `i18n::t`. Add keys to `noro-shared` (`en.ftl` and `ru.ftl`) first.

---

## 3. GPUI UI Engine Rules

GPUI is Zed's GPU-accelerated immediate/retained-mode UI framework.

### 3.1 Pinned Revision
- GPUI is pinned in `Cargo.toml`. **Do NOT upgrade to `main`** — upstream uses unstable compiler features and breaks on the pinned Rust toolchain.

### 3.2 Texture Memory Leak Prevention
- GPUI caches every `RenderImage` in its internal GPU sprite atlas by ID and **never evicts it automatically**.
- Replacing an image in a loop (e.g. skin preview or dynamic animations) without `cx.drop_image(old, None)` leaks GPU and RAM memory (~35 MB/s, previously reaching 10 GB in minutes).
- Always return the old image handle via `cx.drop_image(old, None)` when no other `Arc` references hold it.

### 3.3 Downscaling Images
- Images decoded to RGBA cost double: once in the RAM heap, and once on GPU VRAM.
- Always downscale images to their exact display dimensions before handing them to GPUI.

### 3.4 Headless CI Limitations
- GPUI requires an active GPU and display server. Headless CI cannot run UI rendering tests.
- UI code is compiled and type-checked via `cargo check`; visual testing is performed manually.

---

## 4. Bridge Protocol (IPC)

- Communication between GPUI (UI thread) and Backend (Tokio async thread) is strictly asynchronous over typed `mpsc` channels using `MessageToFE` and `MessageToBE`.
- **Never block the GPUI main thread** with synchronous file I/O, network requests, or `std::thread::sleep`.
- All heavy operations (game file verification, SHA-1 checksumming, HTTP downloads) run on background Tokio threads.

---

## 5. Development & Verification Commands

Before submitting changes, ensure all checks pass:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo fmt --all
```

> [!NOTE]
> CI runs fmt, clippy, and tests on a self-hosted runner. Do not trigger pushes without testing locally first.
