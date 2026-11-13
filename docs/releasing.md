# Releasing

A release is a GitHub Release that carries the ready-built mods, so a player can import them
without installing Rust. The aim is that every release was checked the same way.

## What is supported

A release states which Cinnabar it was tested on. That is the version in
`docs/known-good-host.json` (written by `python scripts\doctor.py --record`, see
[stability.md](stability.md)), plus the pinned `mod-api` revision in `Cargo.toml`. Anything else is
unverified; the release notes should say so.

## Before tagging

1. `cargo fmt --all --check`, both clippy runs and `cargo test --workspace --locked` pass (see
   [CONTRIBUTING.md](../CONTRIBUTING.md)), and so does `python scripts\check-host.py`.
2. Install the build (`python scripts\build-mod.py`) and check in the game, in a creative world
   with a mob nearby, with **Cinnabar's host unchanged since the last `doctor.py --record`**:
   - F8 opens the menu; every page opens and Back returns; changes are still there after a restart.
   - Block outline and Mob hitbox draw, and Cinnabar's own selection is hidden as documented.
   - Counters, crosshair, hotbar keys, armor HUD and effect timers show and move in the HUD editor.
   - Hold **Z**: the zoom eases in and out and **=** / **-** change it.
   - Jump reset shows a timing after a hit.
3. Measure fuel with `python scripts\build-mod.py --fuel-profile` in a busy world, read the
   `fuel_report` (see [architecture.md](architecture.md)), and note the average and peak frame in
   the release notes. Keep the average under about 30,000 of the 200,000 budget. Then rebuild
   without the flag.
4. `python scripts\doctor.py` reports no FAIL. If you tested on a newer Cinnabar than the recorded
   one, confirm in the game first and then run `python scripts\doctor.py --record` and commit
   `docs/known-good-host.json`.
5. Update [features.md](features.md) and the README for anything that changed.

## Tagging

Set the same version in the workspace `Cargo.toml`, commit, then:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The **Release** workflow (`.github/workflows/release.yml`) refuses a tag that does not match the
Cargo version, runs the same gates as CI, builds both components, and publishes
`block-outline.wasm`, `hive-autocs.wasm` and `SHA256SUMS`. Edit the generated notes to add the
tested Cinnabar version and the fuel numbers from step 3.

The installed file must stay `block-outline.wasm`: Cinnabar keys settings and permission grants by
that name.
