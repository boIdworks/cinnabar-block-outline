# Staying stable while Cinnabar keeps updating

Boldy Mods is built against one fixed Cinnabar API revision (the `mod-api` rev pinned in the
workspace `Cargo.toml`), but the Cinnabar you run is replaced all the time. When the mod
misbehaves, the cause is almost always that gap, not new code in this repository.

## Why it drifts

What the machine showed on 2026-11-09:

- Cinnabar was on the **Nightly** channel with **Install updates when I quit** on, so each session
  can start on a different host build. In two days it went from 0.1.31 (build 131) to 0.2.1
  (build 148); upstream `dev` was a further 107 commits ahead of the pinned revision.
- The pin was the 0.1.31 era until 2026-11-14, when the nightly (core 8126f98c9) dropped that API
  revision (`player-state` `read-actors-limited` changed its result type) and the mod went to
  'error'. The pin is now `1b117928` (0.2.1); it built with no source changes. Cinnabar keeps older API revisions loadable and drops one only on a
  breaking change (`crates/mod-api/compatibility-revisions.json`, oldest first), so a mod keeps
  *loading* across most updates. What can still change is **behaviour**: four commits touching
  `crates/mod-api` landed between the pin and the installed build (sprint control, mod block
  breaking, mask actors, hiding the position line), and the host's fuel and frame-callback rules
  were changed in the days before.
- Two failures need different fixes, so tell them apart first: the host **refuses** the mod
  (does not load, is quarantined, loses grants) or the mod **loads but acts differently**.

## When it breaks: run the doctor

```powershell
python scripts\doctor.py
```

It reads only. It reports the state Cinnabar gives the mod, the stored grants, whether the
installed file is the one in `dist\`, whether the host still supports the mod's API revision, the
host commits that touched `crates/mod-api` since the last known-good host, and any quarantine in
the recent logs. A FAIL line says what to do. If it finds nothing, the problem is behaviour, and
the list of changed host commits is where to look.

## Make the host hold still

Cinnabar's own settings, which this repository cannot change for you: **Settings > Updates**,
choose **Beta** instead of Nightly (Beta follows tagged releases) and turn **off** *Install
updates when I quit*. Update on purpose, then:

```powershell
python scripts\doctor.py          # still supported? what changed?
# play a minute with the mod: outline, F8 menu, counters, crosshair, zoom
python scripts\doctor.py --record # remember this host as known good
```

`--record` writes `docs/known-good-host.json` (the host version and commit). Commit it when it
changes. Later runs compare the installed host with it. Only record after you have seen the mod
work in the game, never from a passing doctor alone.

## Updating the pin

Do this only when the doctor says the host dropped the mod's API revision, or when a feature needs
something newer. Do not follow upstream `dev` for its own sake: a newer pin makes the mod need a
newer host (a mod using a newly added capability needs the newer API revision).

1. Read what changed: `compatibility-revisions.json` and the commits to `crates/mod-api`
   between the old and new revision, and `docs/modding` upstream.
2. Change `rev` in **both** `Cargo.toml` (`mod-api`) and `tools/host-check/Cargo.toml` (`ui`).
   `python scripts\doctor.py --self-test` fails if they differ. If upstream asks for a newer
   Rust, change `rust-toolchain.toml` and `rust-version` too.
3. `cargo update -p mod-api` (and `cargo update -p ui` inside `tools/host-check`), then fix
   compile errors in the `guest` glue only; the host-independent modules should not change.
4. Run every check in [CONTRIBUTING.md](../CONTRIBUTING.md), including `scripts\check-host.py`
   and both `build-*.py --no-install` builds (Hive Auto CS shares the pin).
5. Install, play, `python scripts\doctor.py --record`.

## Quarantine

Cinnabar quarantines a mod that traps, overruns its fuel repeatedly or keeps sending refused
patches ([architecture.md](architecture.md#fuel-the-budget-that-shapes-the-design)); the status
is in `%LOCALAPPDATA%\Cinnabar\local-mod.status.json`. After a host update the same code can
cost more or less fuel, so check the start page's fuel line (avg / recent / peak of 200k).
Restart Cinnabar or reload the mod to bring it back, and fix the cause before it recurs.

## Rules that keep the mod loadable

- The installed name stays `block-outline.wasm`. Settings keys are only ever added, each with a
  default, so an old settings file still loads; a bad file falls back to defaults.
- No `unwrap`, `expect`, `panic!` or unchecked indexing in guest code (`panic = "abort"` turns
  one into a trap). A refused or empty host reply switches that one feature off for the frame
  and the rest keeps running.
- Request only the grants a feature needs, and do not use capabilities newer than the pin needs.
