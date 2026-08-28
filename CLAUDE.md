# CLAUDE.md

Essential summary. Full rules in **[INSTRUCTIONS.md](./INSTRUCTIONS.md)**.

## Hard Requirements
- **Keep it Simple.** No unnecessary abstractions or "костыли".
- **≤150 lines per file** — target for new code; **400 is the hard ceiling**.
  Over 150 needs a one-line reason at the top. See INSTRUCTIONS §21.1.
- **One component = one file.**
- **Dark-first**, ATOM style. **Colors from tokens only.**
- **4-pt Grid.** Every dimension must be a multiple of 4.
- **UI is English-only.**
- **Web: Bun only.** Never use npm.
- **GPUI:** Pinned version. Do not move to main.
- **SQLx:** Runtime-checked queries only.

## Commands
```bash
cargo check --workspace
cd web && bun run dev
```
