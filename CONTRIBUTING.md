# Contributing

## Build and check

Install the Rust toolchain and the `wasm32-unknown-unknown` target pinned in `rust-toolchain.toml`
(rustup does this for you inside the repository). Then:

```powershell
python scripts\build-mod.py       # build, pack and install Boldy Mods (add --no-install to only build)
python scripts\build-autocs.py    # the same for Hive Auto CS
python scripts\build-mod.py --fuel-profile   # temporary: also measures fuel per feature (see architecture.md)

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p boldy-mods -p hive-autocs --target wasm32-unknown-unknown --release -- -D warnings
cargo test --workspace --locked
python scripts\check-host.py      # needs: pip install wasmtime
python scripts\jump-reset-report.py --self-test
python scripts\doctor.py --self-test     # also checks both Cinnabar pins match
```

When the mod misbehaves in the game, run `python scripts\doctor.py` (it needs your Cinnabar
install, so CI only runs its self-test) and see [docs/stability.md](docs/stability.md).

CI (`.github/workflows/ci.yml`) runs all of these on every push and pull request. The wasm clippy
run still matters: it checks the code exactly as it ships.

After changing anything the mod sends to the HUD or the menu, run `scripts\check-host.py`. Unit
tests alone do not catch a refused patch; see [docs/architecture.md](docs/architecture.md).

## Where things go

See the module map in [docs/architecture.md](docs/architecture.md). The short version: put logic in
a host-independent module with unit tests, and keep the wasm-only `guest` modules to glue that
calls the host. Keep files focused; split by responsibility when one grows past ~800 lines.

## Rules for changes

- Keep the Rust/WASM component entry point and Cinnabar packaging working.
- Add only the permissions the feature needs, in `mods/boldy-mods/pack/grants.json`.
- Check the result of every host API call and keep frame work within the documented fuel budget
  (200,000 instructions per frame).
- Add dependencies only when necessary, and keep the build command working.
- Boldy Mods must not add operating-system, filesystem, network or WASI access. A personal mod
  runs in a sandbox and talks to the game through documented capabilities only; the packer
  refuses WASI imports. (Hive Auto CS is a packet mod and has its own grants, including `wasi` for
  its log output.)
- Keep the installed file name `block-outline.wasm`: renaming it resets every user's settings and
  permission grants.
- To change the mod card's icon, replace `assets/base-icon.png` (a PNG no larger than 256 x 256
  pixels and 256 KiB; it is embedded when building).
- Update [docs/features.md](docs/features.md) or the other docs when behaviour changes.

## Working with an AI assistant

Before implementing a feature, read the
[Cinnabar modding guide](https://cinnabar.gg/docs/modding/), its relevant guide and API
reference, and a matching example in the sibling `cinnabar\examples\mods` checkout. The `mod-api`
dependency is pinned to a Cinnabar source revision in the workspace `Cargo.toml`; its generated
bindings are the source of truth for available interfaces. Do not rely on guessed API names.

Describe the intended player-facing behaviour, target world, relevant inputs or game state,
restrictions and how to verify it. Ask the assistant to inspect the docs and examples, identify the
minimum grants, implement the feature, and run the build and the checks above.

Personal mods do not have direct world-editing or raw-packet access. If a feature needs a server
guest or trusted packet access, choose the corresponding mod type and follow its separate guide
instead of extending a personal mod. (Hive Auto CS is a packet mod for that reason.)
