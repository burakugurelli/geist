# Geist

Tiny runtime-synthesized interface sounds backed by a dependency-free Rust
core.

The first web vertical slice provides three semantic cues through one public
function:

```ts
import { play } from "@geist/interface-sounds";

play("success");
```

## Current scope

- `press`
- `success`
- `error`
- deterministic mono PCM synthesis in Rust
- lazy WASM and Web Audio initialization
- rendered-buffer caching by cue and sample rate
- no audio assets
- no runtime npm dependencies

Swift, SwiftUI, declarative HTML binding, additional cues, and publishing are
outside this first slice.

## Prerequisites

- stable Rust with the `wasm32-unknown-unknown` target
- `wasm-pack` 0.15 or newer
- Node 22 or newer
- npm 10 or newer

## Verify

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo check --workspace --target wasm32-unknown-unknown
wasm-pack build bindings/wasm \
  --target web \
  --out-dir ../../web/generated \
  --out-name geist_wasm

cd web
npm_config_cache=/tmp/geist-npm-cache npm ci
npm run typecheck
npm test
npm run build
npm ls --omit=dev --depth=0
npm audit
```

## Run the demo

```bash
cd web
npm_config_cache=/tmp/geist-npm-cache npm ci
npm run dev
```

Open the local URL printed by Vite and trigger each cue from a user
interaction.

## Architecture

- `crates/geist-core`: recipes and allocation-free PCM rendering
- `bindings/wasm`: the only Rust crate coupled to `wasm-bindgen`
- `web`: lazy WASM loading, browser audio lifecycle, caching, and playback
- `examples/web`: a consumer of the public `play(cue)` API

The public web entry point exports only `play` and the `CueName` type. Platform
and synthesis details stay behind narrow module boundaries.

[Cuelume](https://github.com/Danilaa1/cuelume) is a product-behavior reference
for this project. Geist's recipes and implementation are original.

## Code style

Modules have one responsibility and communicate through narrow interfaces.
Comments document only non-obvious contracts such as deterministic noise,
allocation boundaries, browser activation timing, and retry behavior.
