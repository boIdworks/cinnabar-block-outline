#!/usr/bin/env python3
"""Build Hive Auto CS (hive-autocs.wasm), a Cinnabar packet mod, and install it.

Pass --no-install to only build, or set CINNABAR_MODS_DIR for another mods folder.
`python scripts/build-autocs.py --recording [FILE]` instead copies the recorder's lines from
Cinnabar's client.log into FILE (default: dist/autocs-recording.txt).
"""

from pathlib import Path
import os
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TAG = "[autocs]"
MORE = "[autocs]+"
# Cinnabar's log appends the stream field to every packet-mod line.
SUFFIX = ' stream="stdout"'


def main() -> None:
    args = sys.argv[1:]
    if args and args[0] == "--recording":
        extract(Path(args[1]) if len(args) > 1 else ROOT / "dist/autocs-recording.txt")
        return
    cargo = [*shlex.split(os.environ.get("CINNABAR_CARGO_WRAPPER", "")), "cargo"]
    target = ROOT / "target"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    run = lambda *command: subprocess.run([*cargo, *command], cwd=ROOT, env=environment, check=True)
    run("build", "--release", "-p", "hive-autocs", "--target", "wasm32-unknown-unknown")
    core = target / "wasm32-unknown-unknown/release/hive_autocs.wasm"
    output = ROOT / "dist/hive-autocs.wasm"
    run("run", "--release", "-p", "hive-autocs-pack", "--", str(core), str(output))
    print(f"Built {output}")
    if "--no-install" not in args:
        install(output)


def install(component: Path) -> None:
    configured = os.environ.get("CINNABAR_MODS_DIR")
    local = os.environ.get("LOCALAPPDATA")
    mods = Path(configured) if configured else Path(local) / "Cinnabar" / "mods" if local else None
    if mods is None or not mods.is_dir():
        print("Cinnabar mods folder not found; import the component through Settings > Mods > Import.")
        return
    target = mods / component.name
    staging = mods / f".{component.name}.installing"
    staging.write_bytes(component.read_bytes())
    try:
        os.replace(staging, target)
    except OSError as error:
        staging.unlink(missing_ok=True)
        print(f"Could not update {target} ({error}); close Cinnabar and build again.")
        return
    print(f"Installed {target}. Cinnabar stops the old copy now and starts this one when you next join a server: leave and rejoin.")


def extract(output: Path) -> None:
    """Collects the recorder's lines from every client log, joining split lines."""
    logs = Path(os.environ.get("LOCALAPPDATA", "")) / "Cinnabar" / "logs"
    lines: list[str] = []
    for log in sorted(logs.glob("client*.log"), key=lambda path: path.stat().st_mtime):
        for raw in log.read_text(encoding="utf-8", errors="replace").splitlines():
            for tag in (MORE, TAG):
                at = raw.find(tag + " ")
                if at < 0:
                    continue
                text = raw[at + len(tag) + 1 :]
                if text.endswith(SUFFIX):
                    text = text[: -len(SUFFIX)]
                if tag == MORE and lines:
                    lines[-1] += text
                else:
                    lines.append(raw[:24] + " " + text)
                break
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {len(lines)} recorder lines to {output}")


if __name__ == "__main__":
    main()
