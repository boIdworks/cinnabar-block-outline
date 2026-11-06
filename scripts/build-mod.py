#!/usr/bin/env python3
"""Build Boldy Mods (block-outline.wasm) as an importable Cinnabar WebAssembly component and install it."""

from pathlib import Path
import os
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    wrapper = shlex.split(os.environ.get("CINNABAR_CARGO_WRAPPER", ""))
    cargo = [*wrapper, "cargo"]
    target = ROOT / "target"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    subprocess.run(
        [
            *cargo,
            "build",
            "--locked",
            "--release",
            "-p",
            "block-outline",
            "--target",
            "wasm32-unknown-unknown",
        ],
        cwd=ROOT,
        env=environment,
        check=True,
    )
    core = target / "wasm32-unknown-unknown/release/block_outline.wasm"
    output = ROOT / "dist/block-outline.wasm"
    subprocess.run(
        [
            *cargo,
            "run",
            "--locked",
            "--release",
            "-p",
            "block-outline-pack",
            "--",
            str(core),
            str(output),
        ],
        cwd=ROOT,
        env=environment,
        check=True,
    )
    print(f"Built {output}")
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
