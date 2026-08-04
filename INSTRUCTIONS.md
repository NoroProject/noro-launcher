# noro-launcher — Project Instructions

> **Single source of truth.** Every agent, AI assistant, and contributor reads this file.
> `AGENTS.md` and `CLAUDE.md` are thin wrappers that point here.

---

## Table of Contents

1. [What Is noro-launcher](#1-what-is-noro-launcher)
2. [High-Level Architecture](#2-high-level-architecture)
3. [Repository Layout](#3-repository-layout)
4. [Core Philosophy](#4-core-philosophy)
5. [Design System — ATOM Style](#5-design-system--atom-style)
6. [4-pt Grid](#6-4-pt-grid)
7. [Typography](#7-typography)
8. [Animation & Motion](#8-animation--motion)
9. [UI/UX Principles](#9-uiux-principles)
10. [Rust Code Style](#10-rust-code-style)
11. [GPUI Desktop Frontend](#11-gpui-desktop-frontend)
12. [Bridge Protocol (IPC)](#12-bridge-protocol-ipc)
13. [Backend Crate (Launcher Logic)](#13-backend-crate-launcher-logic)
14. [Master Server (Axum)](#14-master-server-axum)
15. [Schema Crate (Shared Types)](#15-schema-crate-shared-types)
16. [Web / Admin Panel (Nuxt 3)](#16-web--admin-panel-nuxt-3)
17. [Database Conventions](#17-database-conventions)
18. [Auth & Security Model](#18-auth--security-model)
19. [Game Launch & Sync Flow](#19-game-launch--sync-flow)
20. [Naming Conventions](#20-naming-conventions)
21. [File & Module Size Rules](#21-file--module-size-rules)
22. [Error Handling](#22-error-handling)
23. [Logging & Tracing](#23-logging--tracing)
24. [Development Commands](#24-development-commands)
25. [Definition of Done](#25-definition-of-done)
26. [Anti-Patterns — Never Do This](#26-anti-patterns--never-do-this)

---

## 1. What Is noro-launcher

**noro-launcher** is a custom Minecraft launcher for a private server network.
It replaces the official Mojang launcher with a tailored experience:

- **Desktop app** — a native cross-platform GUI built with GPUI (Rust).
- **Master server** — Axum/PostgreSQL backend that manages users, servers, builds, files, and auth.
- **Web / admin panel** — Nuxt 3 site covering the public landing, player cabinet, and operator admin.
- **Auth chain** — Discord OAuth2 as identity provider → our Bearer token → Yggdrasil for Minecraft auth → authlib-injector in the JVM.
- **Signed manifests** — every build's file list is signed with ed25519; the launcher verifies the signature before launching.
- **Auto-update** — master tracks launcher versions per platform; the launcher self-updates from the master's file store.

### What the launcher does for the player

1. Player opens the launcher, clicks "Sign in with Discord."
2. Browser opens; Discord OAuth completes; launcher receives a token via local callback server.
3. Launcher fetches available servers from master, renders them in the sidebar.
4. Player picks a server, launcher downloads/verifies all files (Java, Minecraft, mods, configs).
5. Launcher runs the game with authlib-injector pointed at the master's Yggdrasil endpoint.
6. Optional mods (per-permission) can be toggled before launch.
7. Console log, sync progress, and notifications appear in the UI in real time.

### What the admin panel does

- **Admin** — manage users, roles, permissions, servers, builds, files, cores, news, launcher versions, capes.
- **Cabinet** — player profile: skin upload, skin 3D preview, cape, stats.
- **Public site** — landing page (future).

---

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Desktop App (noro_launcher binary)                          │
│                                                              │
│  ┌──────────────┐  bridge (mpsc)  ┌───────────────────────┐ │
│  │  frontend    │ ◄─────────────► │      backend          │ │
│  │  (GPUI)      │  MessageToFE/BE │  auth / sync / runner │ │
│  └──────────────┘                 └──────────┬────────────┘ │
└────────────────────────────────────────────── │ ────────────┘
                                                │  HTTPS/WSS
                                    ┌───────────▼───────────┐
                                    │   master server        │
                                    │   (Axum + PostgreSQL)  │
                                    │   + S3 file store      │
                                    └───────────┬────────────┘
                                                │  REST / WS
                                    ┌───────────▼───────────┐
                                    │   web / admin (Nuxt 3) │
                                    └────────────────────────┘
```

**Key boundaries:**

| Boundary | Protocol | Notes |
|---|---|---|
| frontend ↔ backend | `mpsc` channels, typed enums | No serialization overhead — same process |
| backend ↔ master | HTTPS REST + WebSocket | Bearer token in `Authorization` header |
| master ↔ MC server | Yggdrasil HTTP (hasJoined) | Standard Mojang protocol, intercepted by authlib-injector |
| web ↔ master | HTTPS REST | Same Bearer token; Discord auth flow in browser |
| launcher self-update | HTTPS download from master | Signature verified with ed25519 before install |

---

## 3. Repository Layout

```
noro-launcher/
├── Cargo.toml               # workspace root — all [workspace.dependencies] live here
├── crates/
│   ├── schema/              # Shared serde types (no tokio, no axum, no sqlx)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── build.rs     # Build, BuildFile, OptionalMod types
│   │       ├── launcher.rs  # LauncherVersion, LauncherConfig
│   │       ├── news.rs      # NewsItem
│   │       ├── permissions.rs
│   │       ├── server.rs    # ServerEntry, ServerManifest
│   │       ├── user.rs      # UserProfile, NotifLevel
│   │       └── ws_protocol.rs # WS messages master ↔ backend
│   │
│   ├── bridge/              # IPC types crossing the GPUI thread boundary
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── message.rs   # MessageToBackend / MessageToFrontend enums
│   │       ├── modal_action.rs
│   │       ├── handle.rs    # BridgeHandle (sender side)
│   │       ├── quit.rs
│   │       └── serial.rs    # SyncStage, GameLogLevel, ClientSettingsState, OptionalModInfo
│   │
│   ├── backend/             # All launcher logic — runs in Tokio runtime
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backend.rs   # Backend actor — owns all state, routes messages
│   │       ├── backend_handler.rs
│   │       ├── config.rs    # Persistent config (dirs, URLs)
│   │       ├── directories.rs
│   │       ├── persistent.rs # Saved state (token, settings)
│   │       ├── updater.rs
│   │       ├── log_reader.rs
│   │       ├── mod_icon.rs
│   │       ├── signing.rs   # ed25519 manifest verification
│   │       ├── ws_client.rs # WebSocket connection to master
│   │       ├── auth/
│   │       │   ├── mod.rs
│   │       │   ├── discord_oauth.rs # Local HTTP server for OAuth callback
│   │       │   └── token_store.rs
│   │       ├── sync/
│   │       │   ├── mod.rs
│   │       │   ├── file_sync.rs   # Orchestrates full sync
│   │       │   ├── downloader.rs  # Concurrent download pool
│   │       │   └── integrity.rs   # sha1 check
│   │       └── game_runner/
│   │           ├── mod.rs
│   │           ├── args.rs        # JVM + game arg construction
│   │           ├── authlib.rs     # authlib-injector javaagent setup
│   │           ├── classpath.rs
│   │           └── java.rs        # Java binary resolution
│   │
│   ├── frontend/            # GPUI desktop UI
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── theme.rs     # Design tokens — ONLY source of colors
│   │       ├── icons.rs
│   │       ├── assets.rs    # Embedded assets (fonts, images)
│   │       ├── state.rs     # AppState (Model<T> in GPUI)
│   │       ├── login.rs     # Login screen
│   │       ├── pages.rs     # Page router
│   │       ├── skin_preview.rs
│   │       ├── skin_render.rs
│   │       ├── image_loader.rs
│   │       ├── console_model.rs
│   │       ├── console_toolbar.rs
│   │       ├── console_controls.rs
│   │       ├── components/  # Reusable UI atoms
│   │       │   ├── mod.rs
│   │       │   ├── atom_art.rs      # Decorative ATOM ASCII art
│   │       │   ├── badge.rs
│   │       │   ├── button.rs
│   │       │   ├── checkbox.rs
│   │       │   ├── cta_button.rs    # Cream/CTA primary button
│   │       │   ├── icon_button.rs
│   │       │   ├── mod_toggle.rs
│   │       │   ├── pixel_title.rs
│   │       │   ├── progress.rs
│   │       │   └── window_chrome.rs # Custom titlebar + window controls
│   │       └── pages/       # Full screens
│   │           ├── shell.rs          # Root layout (sidebar + content)
│   │           ├── sidebar.rs
│   │           ├── sidebar_server.rs
│   │           ├── sidebar_user.rs
│   │           ├── game.rs           # Server detail page
│   │           ├── game_bar.rs
│   │           ├── game_console.rs
│   │           ├── game_status.rs
│   │           ├── game_sync.rs
│   │           ├── common.rs
│   │           ├── mod_icon.rs
│   │           ├── news.rs
│   │           ├── optional_mods_panel.rs
│   │           ├── profile.rs
│   │           ├── profile_asset.rs
│   │           ├── profile_skin.rs
│   │           ├── server_mods.rs
│   │           ├── server_settings.rs
│   │           ├── settings.rs
│   │           └── toast.rs
│   │
│   ├── master/              # Axum web server
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── state.rs     # AppState (DB pool, config, file store)
│   │       ├── config.rs
│   │       ├── error.rs     # AppError → axum IntoResponse
│   │       ├── signing.rs
│   │       ├── manifest.rs
│   │       ├── api/
│   │       │   ├── mod.rs
│   │       │   ├── launcher.rs   # Launcher-facing REST endpoints
│   │       │   ├── cabinet.rs    # Player cabinet REST
│   │       │   ├── auth/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── discord.rs    # Discord OAuth callback
│   │       │   │   ├── middleware.rs # Auth extractor
│   │       │   │   └── yggdrasil.rs  # Minecraft session protocol
│   │       │   └── admin/        # Admin-only routes
│   │       │       ├── mod.rs
│   │       │       ├── router.rs
│   │       │       ├── users.rs
│   │       │       ├── roles.rs
│   │       │       ├── servers.rs
│   │       │       ├── builds.rs
│   │       │       ├── build_routes.rs
│   │       │       ├── versions.rs
│   │       │       ├── launcher.rs
│   │       │       ├── cores.rs
│   │       │       ├── capes.rs
│   │       │       ├── news.rs
│   │       │       ├── stats.rs
│   │       │       └── tokens.rs
│   │       ├── db/
│   │       │   ├── mod.rs
│   │       │   ├── models.rs    # DB row structs
│   │       │   ├── queries.rs   # sqlx runtime queries
│   │       │   └── capes.rs
│   │       ├── files/
│   │       │   ├── mod.rs
│   │       │   ├── store.rs     # FileStore trait
│   │       │   └── s3.rs        # S3/R2 implementation
│   │       ├── ws/
│   │       │   ├── mod.rs
│   │       │   └── hub.rs       # WebSocket hub for push notifications
│   │       ├── build_importer/
│   │       │   ├── mod.rs
│   │       │   ├── mrpack.rs    # Modrinth .mrpack import
│   │       │   └── curseforge.rs
│   │       ├── launcher_builder/
│   │       │   ├── mod.rs
│   │       │   └── github_watcher.rs
│   │       └── mojang_bootstrap/
│   │           ├── mod.rs
│   │           ├── minecraft.rs   # Vanilla manifest fetch
│   │           ├── fabric.rs
│   │           ├── forge.rs
│   │           ├── java.rs        # Adoptium JDK distribution
│   │           ├── assets.rs      # Asset index + objects
│   │           ├── maven.rs
│   │           └── platform.rs    # OS/arch detection
│   │
│   ├── admin_cli/           # CLI tool for server-side admin tasks
│   └── noro_launcher/       # Binary entry point — spawns tokio + GPUI
│
├── web/                     # Nuxt 3 app
│   ├── app.vue
│   ├── app.config.ts
│   ├── assets/css/
│   │   ├── main.css         # CSS custom properties (--noro-* tokens)
│   │   ├── buttons.css
│   │   ├── surfaces.css
│   │   ├── controls.css
│   │   └── file-manager.css
│   ├── components/
│   │   ├── atom/            # Primitive UI components
│   │   ├── build/           # Build editor panels
│   │   ├── server/          # Server management panels
│   │   ├── file-manager/    # In-browser file manager
│   │   └── admin/           # Admin-specific components
│   ├── composables/         # useXxx hooks
│   ├── pages/               # Nuxt file-based routing
│   └── middleware/
│
└── data/files/              # Local file store (dev) — sharded by first byte of sha1
```

---

## 4. Core Philosophy

These rules override cleverness. When in doubt, choose the simpler path.

### 4.1 Keep It Simple

- Always prefer simple, readable solutions over complex ones.
- Write code that a competent developer can understand in 60 seconds.
- Abstract only when it genuinely pays off (3+ call sites, meaningful reuse).
- Three similar lines of code are better than a premature abstraction.

### 4.2 No Hacks (No костыли)

- Never ship hacky quick fixes that are hard to support later.
- If a proper fix needs a larger scope, **explain the tradeoff and ask** instead of shipping a fragile patch.
- Hacks compound. One костыль today means three костыли next week.

### 4.3 Durable Solutions

- Clear data flow and low coupling.
- State lives in one place; it doesn't replicate.
- Side effects are explicit, never hidden.
- Avoid: unnecessary condition chains, duplicate logic, hidden side effects.

### 4.4 Scope Honesty

- If you discover that the correct fix is bigger than the task, say so.
- Don't silently expand scope. Don't silently shrink it either.
- Ask the user when trade-offs matter.

### 4.5 Performance Is a Feature

- The launcher must feel instant.
- No blocking the UI thread — ever.
- Optimistic updates where safe.
- Downloads run in a concurrent pool, never sequentially.

---

## 5. Design System — ATOM Style

**ATOM** is the visual language: deep navy cosmos, cream CTA, pink-magenta crystal accent.
Dark-first. One theme. No light mode for the launcher (web may follow in the future).

### 5.1 Design Tokens — Rust (`crates/frontend/src/theme.rs`)

**These are the ONLY source of colors in Rust code. Never hardcode hex in components.**

#### Backgrounds

| Token | Hex | Usage |
|---|---|---|
| `BG_WINDOW` | `#0d1b2e` | Root window fill |
| `BG_PANEL` | `#13233d` | Panels, drawers |
| `BG_CARD` | `#172a47` | Cards, list items |
| `BG_CARD_HOV` | `#1f3556` | Card hover state |
| `BG_INPUT` | `#0f2036` | Text inputs, textareas |
| `SIDEBAR` | `#0b1626` | Left sidebar fill |
| `BG_HEADER` | `#0b1626` | Top header bar |
| `OVERLAY` | `#081020` | Modal backdrop |
| `CONTENT_FALLBACK` | `#0a1626` | Content area when no server background |

#### Accents

| Token | Hex | Usage |
|---|---|---|
| `CTA` | `#f3e7b3` | Cream primary button |
| `CTA_HOV` | `#fbf0c4` | Cream button hover |
| `ON_CTA` | `#12233d` | Text on cream button |
| `ACCENT` | `#e85aa5` | Magenta crystal — selection, highlight, focus ring |
| `ACCENT_HOV` | `#f06fb4` | Magenta hover |
| `BLUE` | `#7fb2ff` | Secondary accent, links |

#### Status

| Token | Hex | Usage |
|---|---|---|
| `SUCCESS` | `#7ee0a4` | Green — success, online, good |
| `WARNING` | `#f3c969` | Amber — caution |
| `ERROR` | `#ff6b8b` | Red-pink — errors, danger |

#### Text

| Token | Hex | Usage |
|---|---|---|
| `TEXT_PRIMARY` | `#dbe6ff` | Body text, labels |
| `TEXT_SECONDARY` | `#9fb0d6` | Subtitles, descriptions |
| `TEXT_MUTED` | `#5a6b91` | Placeholder, disabled, timestamps |

#### Borders & Radius

| Token | Value | Usage |
|---|---|---|
| `BORDER` | `#223a55` | Default border color |
| `BORDER_FOCUS` | `= ACCENT` | Focus state border |
| `R_SM` | `4.0 px` | Badges, chips, small inputs |
| `R_MD` | `8.0 px` | Buttons, form fields, small cards |
| `R_LG` | `12.0 px` | Panels, large cards, modals |

#### Fonts

| Token | Value | Usage |
|---|---|---|
| `FONT` | `"Inter"` | All UI body text |
| `FONT_PIXEL` | `"Press Start 2P"` | Logo, large pixel headings |
| `FONT_PIXEL_ALT` | `"Silkscreen"` | Section labels, smaller pixel text |

> **Note:** Pixel fonts do not support Cyrillic. All visible text must be in English.

---

### 5.2 Design Tokens — Web (`web/assets/css/main.css`)

**In web components, use `--noro-*` CSS custom properties only. Never hardcode hex.**

```css
--noro-bg:           #0d1b2e   /* Root background */
--noro-bg-deep:      #07111f   /* Deepest background */
--noro-sidebar:      #091625   /* Sidebar */
--noro-panel:        #13233d   /* Panel surface */
--noro-panel-2:      #182b49   /* Elevated panel */
--noro-input:        #07172a   /* Input background */
--noro-border:       #29466d   /* Default border */
--noro-border-soft:  #1d3558   /* Subtle border */
--noro-text:         #c8d6f0   /* Primary text */
--noro-muted:        #5a6b91   /* Muted/placeholder text */
--noro-cream:        #f3e7b3   /* CTA / primary action */
--noro-cream-hover:  #fff0bd   /* CTA hover */
--noro-on-cream:     #1b2742   /* Text on cream */
--noro-magenta:      #e85aa5   /* Accent / selection */
--noro-blue:         #7fb2ff   /* Secondary accent */
--noro-green:        #79e17f   /* Success */
--noro-amber:        #f2b84b   /* Warning */
--noro-danger:       #ff4f66   /* Error / destructive */
--noro-white:        #ffffff   /* Pure white */
```

---

### 5.3 Component Elevation Model

Three surface levels. Each level gets slightly lighter background:

```
Level 0 — Window / root: BG_WINDOW (#0d1b2e)
Level 1 — Panels / sections: BG_PANEL (#13233d)
Level 2 — Cards / list items: BG_CARD (#172a47)
Level 3 — Inputs / dropdowns: BG_INPUT (#0f2036)
```

Avoid going deeper than level 3. If you need more nesting, your component is too complex — split it.

### 5.4 Sidebar

- Fixed width, fills full height.
- Background: `SIDEBAR` / `--noro-sidebar`.
- User avatar at the bottom, server list above.
- Active server gets a left-edge accent strip in `ACCENT`.
- No hover tooltips on sidebar items — labels are always visible.

### 5.5 Custom Window Chrome

- The launcher has a custom titlebar (`window_chrome.rs` / `WindowChrome`).
- Drag region covers the entire top bar.
- Window controls (close/minimize/maximize) are always visible on the right.
- No native OS titlebar.

---

## 6. 4-pt Grid

Every dimension in the UI snaps to multiples of 4.

```
4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 56, 64, 72, 80, 96, 128 ...
```

This applies to:
- Spacing (padding, margin, gap)
- Component dimensions (width, height)
- Border radius
- Icon sizes
- Offsets and positions

**Exceptions (and only these):**
- Font sizes — use any readable value (15, 14, 13 px etc.)
- Line heights — follow typographic convention

**In practice:**
- Button height: `32px` (compact) / `40px` (default) / `48px` (large)
- Icon size: `16px` / `20px` / `24px`
- Sidebar width: `64px` (icon-only) / `240px` (expanded)
- Card padding: `16px` / `24px`
- Section gap: `8px` / `12px` / `16px`

Never use: `5px`, `7px`, `10px`, `15px`, `18px`, `22px`, `30px`, `45px`.

---

## 7. Typography

### 7.1 Font Stack

| Context | Font | Size floor |
|---|---|---|
| Body text | Inter | 14px |
| Labels, captions | Inter | 13px |
| Minimum readable | Inter | 12px (absolute minimum) |
| Logo / large hero | Press Start 2P | 10–16px |
| Section labels | Silkscreen | 8–10px |

> Body text floor is 14px. Primary body is 15px. Never go below 12px for any readable text.
> Primary actions (buttons) must be ≥ 40px height.

### 7.2 Hierarchy

```
h1: 24px Bold   — Page title (rare in the launcher)
h2: 18px SemiBold — Section heading
h3: 15px SemiBold — Subsection, card title
body: 14–15px Regular — Main content
small: 13px Regular — Captions, secondary
caption: 12px Regular — Timestamps, metadata
```

### 7.3 Text Rules

- No orphans/widows: never leave a single word on the last line of a paragraph.
- Truncate long strings with ellipsis; don't wrap player names.
- Use `TEXT_SECONDARY` for descriptions, not muted — muted is for truly unimportant info.

---

## 8. Animation & Motion

### 8.1 What to Animate

- **Allowed:** `transform`, `opacity` — GPU-accelerated, zero layout cost.
- **Forbidden in hot paths:** `width`, `height`, `top`, `left`, `margin`, `padding`.

### 8.2 Duration

| Type | Duration |
|---|---|
| Micro (hover, focus) | 80–120ms |
| Standard (enter/exit) | 120–200ms |
| Complex (page transition) | 200–280ms |

Never exceed 300ms unless there is a strong UX reason.

### 8.3 Easing

- Enter: `ease-out` — fast start, gentle settle.
- Exit: `ease-in` — quick departure.
- Hover: `linear` at 80ms — feels instant.

### 8.4 Reduced Motion

Always respect `prefers-reduced-motion`. In web CSS:

```css
@media (prefers-reduced-motion: reduce) {
  * { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
}
```

In GPUI, skip animations when the platform accessibility setting is active.

---

## 9. UI/UX Principles

### 9.1 Native & Calm

- Generous spacing — don't crowd the UI.
- Soft rounded corners on panels and cards.
- Clear visual hierarchy: background → surface → elevated.
- The app must feel quiet, focused, and fast.

### 9.2 Desktop-First, Resize-Friendly

- No fixed-pixel layouts that break on resize.
- Flex/grid layouts that adapt gracefully.
- Test at 800×600 minimum and 2560×1440 maximum.
- Sidebar is fixed width; content area fills remaining space.

### 9.3 Friendly & Approachable

- Clear, human-readable copy. No jargon for players.
- Every empty state has an explanation and a next action.
- Every loading state shows meaningful progress (not just a spinner).
- Every error state shows what went wrong and what the user can do.

### 9.4 State Coverage

Every component must handle these states explicitly:

| State | Required |
|---|---|
| Loading | Skeleton or spinner |
| Empty | Empty-state message + optional CTA |
| Error | Readable error + retry action |
| Populated | The happy path |
| Disabled | Visually distinct, with a reason if non-obvious |

### 9.5 Feedback

- Every user action must have immediate feedback (optimistic update, loading indicator, or confirmation toast).
- Destructive actions require confirmation.
- Background operations (download, sync) show progress in the UI.

### 9.6 No Anti-Patterns

- No prop-drilling more than 2 levels — use composables / state.
- No god-components (>150 lines).
- No inline magic numbers — use theme tokens.
- No `any` in TypeScript.
- No unhandled promises — always `.catch()` or `await` in a `try/catch`.
- No fetching in loops — batch or use `Promise.all`.
- No blocking the UI thread in GPUI — all async work runs on the Tokio side.

---

## 10. Rust Code Style

### 10.1 General

- Follow the Rust API Guidelines.
- `rustfmt` on every file. Non-negotiable.
- `clippy` warnings are errors in CI.
- Use `anyhow::Result` for application-level errors.
- Use `thiserror` for library/domain errors that need matching.

### 10.2 Naming

```rust
// Types, traits, enums, variants — PascalCase
struct ServerEntry { ... }
enum SyncStage { CheckingFiles, Done }
trait FileStore { ... }

// Functions, methods, variables, modules — snake_case
fn read_note() -> anyhow::Result<Note> { ... }
let server_id: Uuid = ...;
mod game_runner;

// Constants — SCREAMING_SNAKE_CASE
const MAX_DOWNLOAD_CONCURRENCY: usize = 8;
pub const BG_WINDOW: u32 = 0x0d1b2e;

// Boolean fields/variables — is_, has_, should_, can_ prefix
let is_banned: bool = ...;
struct Mod { is_optional: bool, is_enabled: bool }
```

### 10.3 Error Handling

```rust
// ✅ Propagate with context
let data = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;

// ✅ Domain errors with thiserror
#[derive(thiserror::Error, Debug)]
pub enum SyncError {
    #[error("manifest signature invalid")]
    InvalidSignature,
    #[error("download failed for {path}: {source}")]
    Download { path: String, #[source] source: reqwest::Error },
}

// ❌ Never unwrap in library code
let x = thing.unwrap();  // NO

// ✅ unwrap() only in tests or truly-infallible const contexts
```

### 10.4 Async

```rust
// ✅ Spawn blocking work off the async thread
let result = tokio::task::spawn_blocking(move || expensive_cpu_work()).await?;

// ✅ Concurrent downloads with bounded concurrency
use futures::stream::{self, StreamExt};
stream::iter(files)
    .map(|f| download(f))
    .buffer_unordered(8)
    .collect::<Vec<_>>()
    .await;

// ❌ Never block inside async fn
std::thread::sleep(Duration::from_secs(1)); // NO — use tokio::time::sleep
```

### 10.5 Imports & Modules

```rust
// Standard groups, separated by blank lines:
// 1. std
// 2. external crates
// 3. local crates (schema, bridge, backend)
// 4. super / crate-internal
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

use schema::{BuildFile, ServerEntry};

use crate::config::Config;
```

### 10.6 Documentation

Write doc comments only when the "why" is non-obvious.

```rust
// ✅ Explain a non-obvious invariant
/// MC UUID is deterministically derived from Discord ID via UUID v5.
/// This means the same Discord account always maps to the same MC UUID
/// without storing a separate mapping table.
pub fn mc_uuid_from_discord(discord_id: &str) -> Uuid { ... }

// ❌ Don't explain what the name already says
/// Gets the name
pub fn get_name(&self) -> &str { &self.name }
```

### 10.7 Traits and Generics

- Prefer concrete types over heavy generics in application code.
- Use trait objects (`dyn Trait`) when you have a genuine runtime variant (e.g., `FileStore` → local or S3).
- Don't add generic parameters you don't need today.

---

## 11. GPUI Desktop Frontend

### 11.1 The Framework

GPUI is a GPU-accelerated immediate-mode-ish UI framework from Zed.
It is pinned to a specific revision in `crates/frontend/Cargo.toml`.

> **Do not update GPUI to main.** The API changes frequently. Stick to the pinned commit.

Before writing any GPUI code, **read the pinned revision's source and examples** — the API differs from any tutorial or documentation you might find online.

### 11.2 Key Concepts

| Concept | What it is |
|---|---|
| `Model<T>` | Shared mutable state. Like a Rc<RefCell<T>> but with change notifications. |
| `View<V>` | A rendered UI component. Owns its own state. |
| `cx.notify()` | Marks the current model/view as dirty; schedules a re-render. |
| `cx.spawn(...)` | Runs a future on the async executor. Use for all async work. |
| `div()` | The base layout element — like HTML `<div>`. |
| `render(...)` | Called on every frame for dirty views. Must be pure given state. |

### 11.3 State Pattern

Global app state lives in `state.rs` as `Model<AppState>`.
Components read state by cloning the `Model<AppState>` handle and calling `.read(cx)`.

```rust
pub struct AppState {
    pub page: Page,
    pub user: Option<UserProfile>,
    pub servers: Vec<ServerEntry>,
    pub sync_progress: HashMap<Uuid, SyncProgress>,
    pub notifications: Vec<Notification>,
    // ... etc
}
```

Do not duplicate state. If two components need the same data, they both read from `AppState`.

### 11.4 Component Structure

```rust
pub struct MyComponent {
    state: Model<AppState>,
    // local state only if truly local
}

impl MyComponent {
    pub fn new(state: Model<AppState>, cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self { state })
    }
}

impl Render for MyComponent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        div()
            .bg(rgba(theme::BG_PANEL, 1.0))
            .p_4()
            .child(/* ... */)
    }
}
```

### 11.5 Sending Messages to Backend

```rust
// Get the bridge handle from state
let handle = state.read(cx).bridge.clone();

// Send a message
cx.spawn(|_, _| async move {
    handle.send(MessageToBackend::LaunchServer { server_id, modal_action }).await;
}).detach();
```

Never call `handle.send()` synchronously in render. Always spawn.

### 11.6 Handling Messages from Backend

The frontend listens on a channel. Messages come in via `MessageToFrontend` enum.
The GPUI event loop processes them and calls `cx.update_model(...)` to update `AppState`,
then notifies the affected views.

### 11.7 GPUI Color Helpers

```rust
use gpui::{rgba, Rgba};

// Convert theme constant to GPUI Rgba
fn color(hex: u32) -> Rgba {
    rgba(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8)  & 0xFF) as f32 / 255.0,
        ((hex)       & 0xFF) as f32 / 255.0,
        1.0,
    )
}
```

### 11.8 Layout Rules in GPUI

- Use `.flex()`, `.flex_col()`, `.items_center()`, `.justify_between()` — mirrors Flexbox.
- Use `.p_N()`, `.m_N()`, `.gap_N()` — these take pixel values; keep them on 4-pt grid.
- Widths: `.w_full()`, `.w(px(240.0))`, `.flex_1()`.
- Never use absolute positioning unless unavoidable (e.g., overlays).

---

## 12. Bridge Protocol (IPC)

The bridge is the communication layer between the GPUI thread and the Tokio backend.
It uses `tokio::sync::mpsc` channels — one in each direction.

### 12.1 MessageToBackend

Defined in `crates/bridge/src/message.rs`.

Key messages:

| Message | When |
|---|---|
| `StartDiscordLogin` | User clicks "Sign in" |
| `Logout` | User clicks logout |
| `RequestServerList` | On startup or refresh |
| `OpenServer { server_id }` | User selects a server in sidebar |
| `LaunchServer { server_id, modal_action }` | User clicks Launch |
| `KillGame { server_id }` | User clicks Stop |
| `SetOptionalMods { server_id, enabled }` | User toggles optional mods |
| `SetMemory { min_mb, max_mb }` | Settings change |
| `SetJvmFlags { flags }` | Settings change |
| `InstallUpdate { version, modal_action }` | User confirms update |
| `UploadSkin { bytes }` | Skin upload from launcher |
| `FocusWindow` | Second-instance signal |
| `Quit` | App close |

### 12.2 MessageToFrontend

Key messages:

| Message | When |
|---|---|
| `LoginSuccess { user }` | Discord OAuth completed |
| `LoginFailed { kind }` | OAuth cancelled/rejected/network error |
| `LoggedOut` | Token revoked or logout |
| `ServerList { servers }` | Servers fetched from master |
| `NewsUpdated { items }` | News fetched |
| `ConfigState { ... }` | Full settings sync on startup |
| `OptionalMods { server_id, mods }` | After opening a server |
| `SyncProgress { server_id, stage, done, total, file }` | File sync progress |
| `SyncComplete { server_id }` | Sync done, ready to launch |
| `SyncFailed { server_id, reason }` | Sync error |
| `GameStarted { server_id }` | Game process running |
| `GameStopped { server_id, exit_ok }` | Game process exited |
| `GameLog { server_id, line, level, timestamp }` | Minecraft log line |
| `LauncherUpdateAvailable { version }` | New launcher version on master |
| `AddNotification { text, level }` | Toast notification |
| `ConnectionState { online }` | WS connection to master changed |
| `CloseModal` | Backend completed a modal action |
| `Quit` | Force-quit the app |

### 12.3 ModalAction

Used for backend → frontend "I'm done, close the dialog" signaling.
A `ModalAction` is created by the frontend, passed with the request, and echoed back
in `CloseModal` when the backend finishes. This prevents the frontend from needing
to track which modal corresponds to which operation.

---

## 13. Backend Crate (Launcher Logic)

### 13.1 Architecture

The backend is a single long-running actor (`Backend` struct in `backend.rs`).
It owns all mutable launcher state and processes `MessageToBackend` messages sequentially
(with async sub-tasks for concurrent work like downloads).

### 13.2 Auth Flow (Discord OAuth)

1. Frontend sends `StartDiscordLogin { modal_action }`.
2. Backend starts a local HTTP server on a random port (e.g., `127.0.0.1:44712`).
3. Backend opens the browser to the master's `/auth/discord?port=44712` URL.
4. Master performs Discord OAuth, redirects to `http://127.0.0.1:44712/callback?code=...`.
5. Backend sends code to master's `/auth/discord/token` to exchange for our Bearer token.
6. Backend stores token in OS keyring via `keyring` crate.
7. Backend sends `LoginSuccess { user }` to frontend.

### 13.3 Sync Flow

Triggered by `LaunchServer` or explicit sync:

1. Fetch server manifest from master (`GET /launcher/servers/{id}/manifest`).
2. Verify ed25519 signature on the manifest.
3. Compare local files against manifest (sha1 check).
4. Download missing/changed files concurrently (pool of 8).
5. Apply Forge patches if needed.
6. Remove extra files not in manifest (except unmanaged paths: `saves/`, `screenshots/`, etc.).
7. Send `SyncComplete` → `GameStarted` flow begins.

### 13.4 Game Runner

1. Resolve Java binary from local store (downloaded during sync).
2. Build classpath from manifest's library list.
3. Build JVM args (memory, GC flags, authlib-injector javaagent).
4. Build game args (username, UUID, access token, asset index, ...).
5. Spawn `java` process, pipe stdout/stderr for log reading.
6. Send `GameStarted`, then stream `GameLog` lines until process exits.
7. Send `GameStopped { exit_ok }`.

### 13.5 Persistent State

Two files in the app data directory:
- `config.json` — master URL, global JVM flags, memory, per-server overrides.
- Keyring — Discord Bearer token (OS-level secure storage).

On startup, backend restores from these and sends `ConfigState` to frontend.

### 13.6 Self-Update

1. Master sends `LauncherUpdateAvailable { version }` over WebSocket.
2. Backend forwards to frontend as `LauncherUpdateAvailable`.
3. User clicks "Update" → `InstallUpdate { version, modal_action }`.
4. Backend downloads the new binary, verifies sha256 + ed25519 signature.
5. Replaces binary on disk (platform-specific), restarts.

---

## 14. Master Server (Axum)

### 14.1 State

`AppState` holds:
- `PgPool` — SQLx connection pool to PostgreSQL.
- `Arc<dyn FileStore>` — either local disk or S3/R2.
- `Config` — env-loaded config (URLs, keys, ports).
- `WsHub` — WebSocket hub for push messages to connected launchers.
- ed25519 signing key for manifest signatures.

### 14.2 Route Groups

| Prefix | Auth | Consumers |
|---|---|---|
| `/auth/discord` | None | Launcher (OAuth flow), web |
| `/auth/yggdrasil` | None (MC protocol) | Minecraft server (join/hasJoined) |
| `/launcher/...` | Bearer token | Desktop launcher |
| `/cabinet/...` | Bearer token | Web cabinet |
| `/admin/...` | Bearer token + admin permission | Web admin panel |
| `/ws` | Bearer token | Desktop launcher WebSocket |

### 14.3 Error Handling Pattern

```rust
// Use a domain error type
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
```

### 14.4 File Store

All user-uploaded files (skins, capes, mod icons, launcher binaries) go through `FileStore`.
In development: local directory with sha1-sharded subfolders (`data/files/XX/...`).
In production: S3-compatible (Cloudflare R2).

Files are keyed by sha1. Content-addressed storage — same content = same key.

### 14.5 Manifest Signing

Every published build gets a signed manifest:
1. Build all `BuildFile` entries (path, sha1, size, side, kind).
2. Serialize to canonical JSON.
3. Sign with ed25519 private key (from env `NORO_SIGNING_KEY`).
4. Store signature in `builds.manifest_signature`.
5. Launcher verifies with the hardcoded public key before launch.

### 14.6 Mojang Bootstrap

The master mirrors (and caches in the file store) all Mojang/Fabric/Forge artifacts:
- Vanilla Minecraft client jars and libraries.
- Fabric loader and API.
- Forge installer and patched jars.
- JDK distributions (Adoptium).
- Asset objects from Mojang CDN.

This means players never hit Mojang/Adoptium CDN directly — everything comes from the master.

---

## 15. Schema Crate (Shared Types)

`crates/schema` contains **only** serde-serializable types.

**Zero dependencies on:** tokio, axum, sqlx, GPUI.
Only allowed: `serde`, `serde_json`, `uuid`, `chrono`, `indexmap`.

### Key types

| Module | Types |
|---|---|
| `server.rs` | `ServerEntry`, `ServerManifest`, `BuildManifest`, `ManifestFile` |
| `build.rs` | `Build`, `BuildFile`, `OptionalMod`, `FileKind`, `FileSide` |
| `user.rs` | `UserProfile`, `NotifLevel` |
| `launcher.rs` | `LauncherVersion`, `LauncherConfig` |
| `news.rs` | `NewsItem` |
| `permissions.rs` | `Permission` enum (all known permission strings) |
| `ws_protocol.rs` | `WsMessage` (master → launcher WebSocket push) |

### MC UUID Derivation

```rust
pub const MC_UUID_NAMESPACE: Uuid = /* fixed bytes */;

pub fn mc_uuid_from_discord(discord_id: &str) -> Uuid {
    Uuid::new_v5(&MC_UUID_NAMESPACE, format!("Discord:{discord_id}").as_bytes())
}
```

Same Discord ID → same MC UUID → no mapping table needed.

---

## 16. Web / Admin Panel (Nuxt 3)

### 16.1 Stack

- **Nuxt 3** + **Vue 3** Composition API.
- **Nuxt UI** (v3) for base components — extended/themed with `--noro-*` tokens.
- **Tailwind CSS** (via Nuxt UI's Tailwind integration).
- **Bun** — only package manager. Never use npm or yarn.

### 16.2 Component Rules

```
components/atom/       — Primitive atoms: Button, Input, Modal, Select
components/build/      — Build editor panels (≤1 panel per file)
components/server/     — Server management panels
components/file-manager/  — File manager sub-components
components/admin/      — Admin-specific (UserPermissions, UserRoles)
```

- One component = one file.
- Components files: `PascalCase.vue`.
- Composables: `use-kebab-case.ts`.
- Other TS files: `kebab-case.ts`.

### 16.3 Composable Pattern

```typescript
// composables/useApi.ts
export function useApi() {
  const { $fetch } = useNuxtApp()

  async function getServers(): Promise<ServerEntry[]> {
    return $fetch('/api/launcher/servers')
  }

  return { getServers }
}
```

One composable per domain. Don't put everything in `useApi`.

### 16.4 State

Use Nuxt's `useState` for shared reactive state across components.
Local component state: `ref` / `reactive`.

Do not prop-drill more than one level. If two sibling components need the same data,
lift it to a composable or `useState`.

### 16.5 Pages Structure

```
pages/
  index.vue              # Landing (public)
  cabinet/
    index.vue            # Player profile
  admin/
    index.vue            # Dashboard
    users/[id].vue       # User detail
    servers/[id]/
      index.vue          # Server settings
      build/[bid].vue    # Build editor
    news/
      index.vue
      [id].vue
    roles/
      index.vue
      [id].vue
    capes.vue
    cores.vue
    launcher/
      index.vue
      tokens.vue
```

### 16.6 CSS Rules

- All colors from `--noro-*` custom properties.
- All spacing on 4-pt grid.
- Use Tailwind utility classes where available.
- Write custom CSS only for things Tailwind can't express.
- Never inline `style=""` with hardcoded colors.

### 16.7 TypeScript Rules

- Strict mode — no `any`.
- All API responses typed (import from schema types or local `types/` dir).
- Use `defineProps<{ ... }>()` with explicit types — no runtime prop validators.
- Use `defineEmits<{ ... }>()` with explicit event types.

---

## 17. Database Conventions

### 17.1 SQLx Runtime Queries

Use `sqlx::query()` / `sqlx::query_as()` — **not** `query!` macros.
Runtime-checked queries don't require a live DB at compile time, making CI simpler.

```rust
// ✅ Runtime query
let user = sqlx::query_as::<_, DbUser>(
    "SELECT * FROM users WHERE discord_id = $1"
)
.bind(discord_id)
.fetch_optional(&pool)
.await?;

// ❌ Compile-time macro (requires live DB in CI)
let user = sqlx::query_as!(DbUser, "SELECT * FROM users WHERE discord_id = $1", discord_id)
    .fetch_optional(&pool)
    .await?;
```

### 17.2 Schema Conventions

- All primary keys: `UUID` generated with `gen_random_uuid()`.
- All timestamps: `TIMESTAMPTZ NOT NULL DEFAULT NOW()`.
- Soft deletes: use `active BOOLEAN NOT NULL DEFAULT TRUE` where needed. Never hard delete user data.
- JSON columns: `JSONB` (not `JSON`).
- Permissions stored as `TEXT` or `TEXT[]` — not integers.
- Foreign keys always `ON DELETE CASCADE` unless there's a strong reason not to.

### 17.3 Migrations

One file per migration. Filename: `NNNN_description.sql`.
Migrations are forward-only — no down migrations.

If you need to undo a migration, write a new forward migration.

### 17.4 DB Models vs Schema Types

Two separate structs:

```rust
// In master/src/db/models.rs — the DB row
pub struct DbUser {
    pub id: Uuid,
    pub discord_id: String,
    pub mc_uuid: Uuid,
    // ... exact column names
}

// In schema/src/user.rs — the API/IPC type
pub struct UserProfile {
    pub id: Uuid,
    pub discord_username: String,
    pub mc_username: String,
    // ... shaped for consumers
}
```

Convert between them with `From<DbUser> for UserProfile` or explicit mapping functions.
Do not expose raw DB models to the API response or bridge.

---

## 18. Auth & Security Model

### 18.1 Identity Chain

```
Discord OAuth2
    ↓
Master issues Bearer token (UUID, stored in oauth_sessions)
    ↓
Launcher/Web sends Bearer in Authorization header
    ↓
Master issues MC access token (UUID, stored in mc_sessions)
    ↓
Launcher injects as Yggdrasil token into JVM via authlib-injector
    ↓
Minecraft server calls master /auth/yggdrasil/hasJoined
```

### 18.2 Permissions

Permissions are strings (e.g., `"admin"`, `"optional_mod.beta_mods"`, `"server.restricted"`).
A user has permissions from:
1. Their roles (`role_permissions`).
2. Direct grants (`user_permissions`).

The launcher receives the merged permission set in `UserProfile.permissions`.
Optional mods marked `limited: true` are only accessible if the user has the matching permission.

### 18.3 Manifest Security

Every published build manifest is signed with ed25519.
The launcher's public key is compiled in at build time (env `NORO_SIGNING_PUBKEY`).
A corrupted or tampered manifest → launch refused.

The DEV seed (`DEV_SIGNING_SEED`) is public in source code — **never use it in production**.

### 18.4 Admin Tokens

Server-side admin operations (CI, `admin_cli`) use admin tokens stored as Argon2 hashes.
Tokens carry a permission set, not full admin access.

### 18.5 CORS

Admin and launcher APIs have strict CORS — only the configured frontend origins.
The Yggdrasil endpoint has no CORS restriction (Minecraft server calls it server-side).

---

## 19. Game Launch & Sync Flow

### 19.1 States

```
Idle → Opening → Syncing → Ready → Launching → Running → Stopped
                    ↓
                SyncFailed
```

### 19.2 Sync Stages (in order)

| Stage | Description |
|---|---|
| `CheckingFiles` | Comparing local files to manifest |
| `DownloadingJava` | Fetching JDK if not present |
| `DownloadingMinecraft` | Client jar + libraries |
| `DownloadingLibraries` | Fabric/Forge libraries |
| `DownloadingAssets` | Sound, textures, languages |
| `DownloadingMods` | Mods from build |
| `ApplyingForgePatches` | Forge binary patches |
| `Cleaning` | Removing extra files |
| `Done` | Ready |

All stages report `{ done, total, file }` for granular progress display.

### 19.3 Unmanaged Paths

These are never touched by the sync (not deleted even if not in manifest):
```json
["saves/", "screenshots/", "options.txt", "logs/", "crash-reports/"]
```

Plus `user_managed_paths` from the build config (per-server custom exclusions).

### 19.4 Optional Mods

Optional mods are part of the build's file list but marked `optional: true`.
The launcher downloads them during sync but places them outside the active `mods/` folder
(or uses a sub-folder) and only copies them into `mods/` when enabled.

The player's selection is persisted locally per server.

---

## 20. Naming Conventions

### 20.1 Rust

| Kind | Convention | Example |
|---|---|---|
| Types, traits, enums, variants | `PascalCase` | `ServerEntry`, `SyncStage::Done` |
| Functions, methods, modules, variables | `snake_case` | `download_file`, `game_runner` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_CONCURRENCY`, `BG_WINDOW` |
| Booleans | `is_`, `has_`, `should_`, `can_` | `is_banned`, `has_permission` |
| Files | `snake_case.rs` | `game_runner.rs`, `file_sync.rs` |

### 20.2 TypeScript / Vue

| Kind | Convention | Example |
|---|---|---|
| Components | `PascalCase.vue` | `BuildEditor.vue`, `FilesPanel.vue` |
| Composables | `use-kebab-case.ts` | `use-build-editor.ts` |
| Other files | `kebab-case.ts` | `api-types.ts` |
| Variables, functions | `camelCase` | `serverList`, `fetchBuilds` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_FILE_SIZE` |
| Booleans | `is`, `has`, `should`, `can` | `isLoading`, `hasPermission` |
| Props | `camelCase` | `serverId`, `isAdmin` |
| Emits | `kebab-case` | `update:modelValue`, `file-selected` |

### 20.3 Component Names

Use plain, obvious nouns:

```
✅  Badge, Card, Button, Sidebar, Modal, Input, Select
✅  FilesPanel, BuildEditor, ServerCard, UserRoles
❌  Shell, Hero, Widget, Wrapper, Container, Handler
```

### 20.4 API Routes

Lowercase, hyphen-separated, nouns:

```
GET    /launcher/servers
GET    /launcher/servers/:id/manifest
POST   /admin/servers
PATCH  /admin/servers/:id
DELETE /admin/builds/:id/files/:fileId
```

---

## 21. File & Module Size Rules

### 21.1 The 150-Line Rule

**Every file must be ≤ 150 lines.**

This is a hard requirement, not a suggestion.
If a file grows past 150 lines:
1. Extract a sub-module.
2. Extract a helper function or struct into its own file.
3. Ask: "Is this component doing too much?"

### 21.2 One Component = One File

- Each GPUI view/component lives in its own `.rs` file.
- Each Vue component lives in its own `.vue` file.
- Don't put two components in one file even if they're small.

### 21.3 Module Organization

When a module grows, extract to a directory:

```
// Before
mod sync;  // sync.rs (120 lines, getting crowded)

// After splitting
mod sync;  // sync/
           //   mod.rs       (re-exports)
           //   file_sync.rs (orchestration)
           //   downloader.rs (download pool)
           //   integrity.rs  (hash checking)
```

---

## 22. Error Handling

### 22.1 Rust

- Use `anyhow::Result<T>` in application/binary code.
- Use `thiserror` for domain errors that need pattern matching by callers.
- Always add context with `.context("what was being attempted")`.
- Log errors at the point they're first handled; don't re-log on every propagation.
- Don't panic in library code. Panics are acceptable only in `main` for unrecoverable startup failures.

### 22.2 TypeScript / Vue

```typescript
// ✅ Always handle async errors
async function loadServers() {
  try {
    servers.value = await api.getServers()
  } catch (err) {
    error.value = 'Failed to load servers'
    console.error(err)
  }
}

// ❌ Unhandled promise
fetchServers() // NO — if it rejects, it's silently lost
```

- Show user-facing error messages in the UI.
- Log technical details to the console for debugging.
- Use toast notifications for background operation failures.

### 22.3 User-Facing Error Messages

- Always in English (pending i18n).
- Human-readable — no stack traces, no internal error codes.
- Include a next action when possible: "Check your internet connection and try again."
- Technical details in console only.

---

## 23. Logging & Tracing

### 23.1 Rust (tracing crate)

```rust
use tracing::{debug, error, info, warn};

// ✅ Structured fields
info!(server_id = %id, "sync started");
warn!(file = %path, expected = sha1_expected, got = sha1_actual, "hash mismatch");
error!(error = %e, "download failed");

// ✅ Debug for verbose dev-only info
debug!(endpoint = %url, "fetching manifest");

// ❌ String formatting in structured fields
info!("sync started for {}", id.to_string()); // prefer structured
```

Log levels:
- `error` — something broke; user impact.
- `warn` — unexpected but recoverable.
- `info` — notable lifecycle events (login, sync start/complete, game start/stop).
- `debug` — verbose operational details (per-file download, per-message IPC).

### 23.2 Frontend Game Console

The game console (`game_console.rs`) displays Minecraft log output.
Lines are classified as `GameLogLevel::Info / Warn / Error` and color-coded:
- `Info` — `TEXT_SECONDARY`
- `Warn` — `WARNING`
- `Error` — `ERROR`

---

## 24. Development Commands

### 24.1 Rust

```bash
# Type-check entire workspace (fastest feedback)
cargo check --workspace

# Build
cargo build --workspace
cargo build --release --package noro_launcher

# Run master server
cargo run --package master

# Run the launcher
cargo run --package noro_launcher

# Clippy (must pass)
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
```

### 24.2 Web

```bash
cd web

# Install dependencies (Bun only — never npm)
bun install

# Dev server
bun run dev

# Type check
bun run typecheck

# Build
bun run build

# Preview production build
bun run preview
```

### 24.3 Database

```bash
# Run migrations (sqlx-cli)
sqlx migrate run --database-url $DATABASE_URL

# Create new migration
sqlx migrate add description_of_change
```

### 24.4 Environment Variables (Master)

See `.env.example`. Key variables:

```
DATABASE_URL=postgresql://...
NORO_SIGNING_KEY=<ed25519 private key PEM — production only>
NORO_SIGNING_PUBKEY=<ed25519 public key — must match compiled-in launcher key>
DISCORD_CLIENT_ID=
DISCORD_CLIENT_SECRET=
DISCORD_REDIRECT_URL=
MASTER_URL=https://...
S3_BUCKET=
S3_ENDPOINT=
S3_REGION=
S3_ACCESS_KEY_ID=
S3_SECRET_ACCESS_KEY=
ADMIN_TOKEN=<initial bootstrap token>
```

### 24.5 GPUI Note

GPUI requires a GPU and a display. It cannot run in headless CI.
Compile and type-check with `cargo check`; visual testing is manual.

---

## 25. Definition of Done

Before marking any task complete, verify:

### Rust / Backend / Master

- [ ] Read relevant GPUI source/examples before writing any GPUI UI code.
- [ ] Every touched `.rs` file is ≤ 150 lines.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] `cargo fmt --all` applied.
- [ ] No `unwrap()` in non-test, non-infallible code.
- [ ] Errors have context messages.
- [ ] All async work is off the UI thread.
- [ ] New DB queries use `sqlx::query()` / `sqlx::query_as()` (no compile-time macros).
- [ ] New DB columns/tables have a migration.
- [ ] No hardcoded hex colors — use `theme::*` constants.
- [ ] All dimensions on the 4-pt grid.

### Web / Frontend

- [ ] Every touched `.vue` / `.ts` file is ≤ 150 lines.
- [ ] `bun run typecheck` passes.
- [ ] `bun run build` passes.
- [ ] No `any` in TypeScript.
- [ ] No hardcoded hex — use `--noro-*` CSS custom properties.
- [ ] All spacing / dimensions on 4-pt grid.
- [ ] Loading / empty / error states handled for all data fetches.
- [ ] Tested at 1280×800 and 1920×1080.

### Both

- [ ] Solution is the **simplest** that works — no unnecessary abstractions.
- [ ] All visible text is English.
- [ ] No inline magic numbers or colors.
- [ ] Loading, empty, and error states are handled.
- [ ] If the fix touched auth or permissions — manually tested the auth flow.

---

## 26. Anti-Patterns — Never Do This

### 26.1 Code Structure

```
❌ Files > 150 lines
❌ Two components in one file
❌ God-components that do everything
❌ Premature abstraction (DRY before you have 3+ real use cases)
❌ Deep prop drilling (>2 levels)
❌ Duplicating state that already lives in AppState
❌ Importing backend types directly into frontend (use bridge/schema only)
```

### 26.2 Colors & Design

```
❌ Hardcoded hex in Rust components: .bg(rgba(0x0d1b2e_u32, 1.0)) directly
❌ Hardcoded hex in CSS: color: #e85aa5 — use var(--noro-magenta)
❌ Dimensions not on 4-pt grid: padding: 5px, height: 45px
❌ Fonts below 12px
❌ Primary action buttons below 40px height
❌ Cyrillic text in the UI (pixel fonts don't support it)
```

### 26.3 Async & Threading

```
❌ Blocking the GPUI render thread: std::thread::sleep, blocking I/O
❌ spawn_blocking inside GPUI render — defer to backend
❌ Unhandled async errors (fire-and-forget .await without error handling)
❌ Sequential downloads — always use concurrent pool
❌ Fetching in a loop: for id in ids { fetch(id) } — use batch API or join_all
```

### 26.4 Database

```
❌ Compile-time query! macros (use runtime sqlx::query instead)
❌ Raw string interpolation in queries (SQL injection)
❌ SELECT * in production queries — always name columns
❌ Missing indexes on foreign keys or frequently-filtered columns
❌ Storing secrets in plain text (use argon2 for passwords, never store tokens unhashed)
```

### 26.5 Security

```
❌ DEV_SIGNING_SEED in production
❌ Skipping manifest signature verification before launch
❌ Trusting client-supplied UUIDs or usernames for MC auth
❌ Admin endpoints without permission checks
❌ CORS wildcard (*) on admin/launcher endpoints
```

### 26.6 Dependencies

```
❌ Updating GPUI to main — it's pinned for a reason
❌ Using npm or yarn in the web project — Bun only
❌ Adding heavy dependencies for simple tasks (lodash for array operations, moment for date formatting)
❌ Vendoring generated files into the repo unnecessarily
```

### 26.7 UI Copy & Text

```
❌ Cyrillic in visible UI text (pixel fonts don't support it)
❌ Jargon in player-facing strings ("UUID", "Yggdrasil", "ed25519")
❌ No feedback after a user action
❌ Missing empty/loading/error states
❌ Single word orphaned on last line of a paragraph
```

---

*Last updated: June 2026. Maintained by the noro-launcher team.*
*This file is the single source of truth. If CLAUDE.md or AGENTS.md conflict with this file, this file wins.*
