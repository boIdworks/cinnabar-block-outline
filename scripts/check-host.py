#!/usr/bin/env python3
"""Check every HUD and menu document and patch Boldy Mods sends against Cinnabar's own admission.

Unit tests check that the mod writes what it means to; this checks that Cinnabar accepts it. A
rejected HUD patch draws nothing, and a mod that keeps sending them can be quarantined, so run
this after changing anything the mod sends to the HUD or the menu.

It builds `tools/host-check` (Cinnabar's `ui` crate at the pinned revision) to wasm, has the
`host_dump` test write the documents and patches, and feeds them to it in order, as
`hud.set-surface`, `hud.update-surface`, the panel bundle and `panel.update-content` would
receive them. Needs the `wasmtime` Python package (`pip install wasmtime`).
"""

from pathlib import Path
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    try:
        from wasmtime import Engine, Instance, Module, Store
    except ImportError:
        print("check-host needs the wasmtime package: pip install wasmtime", file=sys.stderr)
        return 2
    target = ROOT / "target"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    checker = ROOT / "tools/host-check"
    subprocess.run(
        ["cargo", "build", "--release", "--target", "wasm32-unknown-unknown"],
        cwd=checker,
        env=environment,
        check=True,
    )
    dump = target / "host-dump"
    subprocess.run(
        ["cargo", "test", "--lib", "host_dump", "--", "--ignored", "--quiet"],
        cwd=ROOT,
        env=dict(environment, BOLDY_DUMP_DIR=str(dump)),
        check=True,
    )

    engine = Engine()
    store = Store(engine)
    wasm = target / "wasm32-unknown-unknown/release/host_check.wasm"
    exports = Instance(store, Module.from_file(engine, str(wasm)), []).exports(store)
    memory = exports["memory"]

    def admit(kind: str, data: bytes) -> str | None:
        pointer = exports["alloc"](store, len(data))
        memory.write(store, data, pointer)
        if exports[kind](store, pointer, len(data)) == 0:
            return None
        at, length = exports["error_ptr"](store), exports["error_len"](store)
        return bytes(memory.read(store, at, at + length)).decode("utf-8", "replace")

    rejected = 0
    files = sorted(dump.iterdir())
    for path in files:
        # Files are named `NNN-kind-label.json`, kind one of the exports above.
        kind = path.name.split("-")[1]
        error = admit(kind, path.read_bytes())
        if error is not None:
            rejected += 1
            print(f"REJECTED {path.name}: {error}")
    print(f"{len(files) - rejected} of {len(files)} documents and patches accepted")
    return 1 if rejected else 0


if __name__ == "__main__":
    sys.exit(main())
