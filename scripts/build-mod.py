#!/usr/bin/env python3
"""Build Boldy Mods as an importable Cinnabar WebAssembly component and install it.

The crate is `boldy-mods`, but the component keeps the file name `block-outline.wasm` it has always
been installed under: Cinnabar keys a mod's settings file and permission grants by that name, so
renaming it would reset every existing install. The name shown in the game is "Boldy Mods"."""

from pathlib import Path
import os
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
# The installed file name (see the module docs); not the crate name.
INSTALL_NAME = "block-outline.wasm"


def main() -> None:
    wrapper = shlex.split(os.environ.get("CINNABAR_CARGO_WRAPPER", ""))
    cargo = [*wrapper, "cargo"]
    target = ROOT / "target"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    # `--fuel-profile` builds with the per-feature fuel profile, which costs fuel itself: it is for
    # measuring, so rebuild without it before keeping the mod installed.
    profile = ["--features", "fuel-profile"] if "--fuel-profile" in sys.argv[1:] else []
    subprocess.run(
        [
            *cargo,
            "build",
            "--locked",
            "--release",
            "-p",
            "boldy-mods",
            *profile,
            "--target",
            "wasm32-unknown-unknown",
        ],
        cwd=ROOT,
        env=environment,
        check=True,
    )
    core = target / "wasm32-unknown-unknown/release/boldy_mods.wasm"
    output = ROOT / "dist" / INSTALL_NAME
    subprocess.run(
        [
            *cargo,
            "run",
            "--locked",
            "--release",
            "-p",
            "boldy-mods-pack",
            "--",
            str(core),
            str(output),
        ],
        cwd=ROOT,
        env=environment,
        check=True,
    )
    print(f"Built {output}")
    if profile:
        print("This is a fuel-profiling build: rebuild without --fuel-profile for everyday use.")
    if "--no-install" not in sys.argv[1:]:
        install(output)


def install(component: Path) -> None:
    """Replaces the installed copy in Cinnabar's mods folder, keeping its settings file."""
    configured = os.environ.get("CINNABAR_MODS_DIR")
    local = os.environ.get("LOCALAPPDATA")
    mods = Path(configured) if configured else Path(local) / "Cinnabar" / "mods" if local else None
    if mods is None or not mods.is_dir():
        print("Cinnabar mods folder not found; import the component through Settings > Mods > Import.")
        return
    retire_old_name(mods, component.stem)
    target = mods / component.name
    staging = mods / f".{component.name}.installing"
    staging.write_bytes(component.read_bytes())
    try:
        os.replace(staging, target)
    except OSError as error:
        staging.unlink(missing_ok=True)
        print(f"Could not update {target} ({error}); close Cinnabar and build again.")
        return
    print(f"Installed {target}; restart Cinnabar (or reload the mod) to use it.")


def retire_old_name(mods: Path, stem: str) -> None:
    """Moves an install from before the rename to the new name, so one copy runs."""
    old = mods / "mod-template.wasm"
    if not old.exists():
        return
    old_settings = mods / "mod-template.settings.json"
    new_settings = mods / f"{stem}.settings.json"
    if old_settings.exists() and not new_settings.exists():
        old_settings.replace(new_settings)
        print(f"Moved your settings to {new_settings}")
    try:
        old.unlink()
        old_settings.unlink(missing_ok=True)
        print(f"Removed the old {old.name}; allow Boldy Mods' permissions again in Cinnabar.")
    except OSError as error:
        print(f"Could not remove {old} ({error}); close Cinnabar and build again.")


if __name__ == "__main__":
    main()
