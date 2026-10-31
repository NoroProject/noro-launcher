# Local builds

`noro-build-agent` builds launcher releases on machines you own — a Mac for
both macOS targets, a Windows box for Windows — and uploads them to the
master. Nothing goes through GitHub: the master stores the source archive,
hands each target to an agent that can build it, and runs what comes back
through the same pipeline as a GitHub release (address stamping, Authenticode
or DMG signing, ed25519, file store).

```
   your working copy ──submit──▶ master ◀──claim/upload── agent (macOS)
                                   ▲
                                   └────claim/upload───── agent (Windows)
```

## Setting up an agent

Once per machine:

1. Install Rust through **rustup** (the toolchain in `rust-toolchain.toml` is
   fetched on the first build) and the platform's build tools: Xcode command
   line tools on macOS, Visual Studio Build Tools with the C++ workload on
   Windows.
2. Get this repository onto the machine once and build the agent:
   `cargo build --release -p build_agent`. Copying the binary from another
   machine of the same OS works too — the agent doesn't need the checkout
   after that; every build comes from the archive the master hands out.
3. Issue an admin token with `noro.admin.launcher.publish` (plus
   `noro.admin.launcher.deploy` if you'll use `--deploy`).

Run it:

```sh
# macOS / Linux
export NORO_MASTER_URL=https://api.example.com
export NORO_ADMIN_TOKEN=noro_at_...
noro-build-agent run
```

```powershell
# Windows
$env:NORO_MASTER_URL = "https://api.example.com"
$env:NORO_ADMIN_TOKEN = "noro_at_..."
noro-build-agent.exe run
```

By default an agent builds what its host builds natively: both Apple targets
on a Mac, `x86_64-pc-windows-msvc` on Windows, the host target on Linux.
`--targets` overrides that; a Linux target on a non-Linux host goes through
`cross` and needs Docker. `--name` sets the name shown in the admin panel
(the host name by default), `--work` the build directory
(`~/.noro-build-agent` by default). The `target/` directory there survives
between builds, so only the first one is cold.

Optional, read from the agent's environment:

| Variable | Effect |
|---|---|
| `NORO_SENTRY_DSN` | baked into the binaries, as in CI |
| `SENTRY_ORG`, `SENTRY_AUTH_TOKEN` | debug files uploaded with `sentry-cli` |
| `MACOS_SIGN_IDENTITY` | Developer ID for `codesign`; ad-hoc otherwise |

The token and `NORO_MASTER_URL` are removed from the build's environment: a
compile-time master address would become the binary's fallback, and the master
stamps the real one in anyway.

## Building

From the working copy you want to ship:

```sh
noro-build-agent submit                 # every target an online agent offers
noro-build-agent submit --targets aarch64-apple-darwin,x86_64-pc-windows-msvc
noro-build-agent submit --deploy core   # make the new core current when done
```

`submit` packs the working copy as it is — uncommitted changes included,
ignored files (and with them `.cargo/config.toml`) left out — tags it
`v<version from Cargo.toml>` and follows the build log until the job closes.
`--no-wait` returns right after queuing.

`--deploy core` makes core current on every platform once all targets
succeed; `--deploy all` does the installers too. Without it, roll out from the
admin panel as usual. Installers are deliberately not deployed by default:
SmartScreen reputation is tied to the installer's hash.

The admin panel shows connected agents under **Releases → Launcher**, and the
build log lists each target with its status and agent. A running local build
can be cancelled there or with `noro-admin launcher cancel <job_id>`.

If an agent goes quiet for ten minutes, its target is handed to another agent
that can build it. An agent that comes back to a target it no longer owns
stops building it.
