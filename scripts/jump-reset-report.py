#!/usr/bin/env python3
"""Reads Boldy Mods' recorded hits and says whether the jump reset threshold fits them.

The `Reset!` verdict compares the knock left a tick after a hit with `jump_reset_carry` (0.08
blocks per tick by default). That default was set from a single recorded reset, so it is a guess.
Play some fights with the Jump reset tool on, then run this:

    python scripts/jump-reset-report.py [path\\to\\block-outline.settings.json]

Without a path it reads the installed settings file (`CINNABAR_MODS_DIR`, else
`%LOCALAPPDATA%\\Cinnabar\\mods`). It looks at the hits where you were on the ground (a jump cannot
reset a hit taken in the air), splits them into those with a jump inside the verdict's window and
those without, and recommends a threshold only if the knock-left values of the jumped hits fall
into two clearly separate groups. With too little data, or no clean split, it says so instead of
inventing a number.

    python scripts/jump-reset-report.py --self-test
"""

from pathlib import Path
import json
import os
import sys

DEFAULT_WINDOW_MS = 75
DEFAULT_CARRY = 0.08
# The verdict only calls a jump a reset when it is within twice the perfect window of the hit.
WINDOW_FACTOR = 2
# Fewest grounded hits and in-window jumps worth drawing a conclusion from.
MIN_GROUNDED = 10
MIN_JUMPED = 4
# The smallest gap (blocks per tick) between the two groups that counts as a clean split.
MIN_GAP = 0.03


def parse_log(text):
    """Rows of `(offset_ms or None, carried, grounded or None, speed)`; `carried` and `speed` in
    blocks per tick. Rows that do not parse are skipped, as the mod does."""
    rows = []
    for part in text.split(";"):
        fields = part.split(",")
        if len(fields) != 4:
            continue
        offset, carried, grounded, speed = fields
        try:
            carried = int(carried) / 1000
            speed = int(speed) / 1000
            offset = int(offset) if offset != "x" else None
        except ValueError:
            continue
        rows.append((offset, carried, {"1": True, "0": False}.get(grounded), speed))
    return rows


def analyse(rows, window_ms, threshold):
    """A dict describing what the recorded hits say about `threshold`."""
    reach = window_ms * WINDOW_FACTOR
    grounded = [r for r in rows if r[2] is not False]
    jumped = [r for r in grounded if r[0] is not None and abs(r[0]) <= reach]
    other = [r for r in grounded if r not in jumped]
    result = {
        "hits": len(rows),
        "grounded": len(grounded),
        "jumped": len(jumped),
        "other": len(other),
        "threshold": threshold,
        "counted": sum(1 for r in jumped if r[1] <= threshold),
        "recommendation": None,
        "note": "",
    }
    if len(grounded) < MIN_GROUNDED or len(jumped) < MIN_JUMPED:
        result["note"] = (
            f"Not enough data: need at least {MIN_GROUNDED} hits on the ground and {MIN_JUMPED} "
            f"with a jump within {reach} ms of the hit (have {len(grounded)} and {len(jumped)})."
        )
        return result
    values = sorted(r[1] for r in jumped)
    gaps = [(values[i + 1] - values[i], i) for i in range(len(values) - 1)]
    gap, at = max(gaps)
    if gap < MIN_GAP:
        result["note"] = (
            f"No clean split: the knock left after your jumps runs from {values[0]:.3f} to "
            f"{values[-1]:.3f} with no gap of {MIN_GAP} or more, so either every jump reset or "
            "none did. Keep playing and try some jumps that are clearly early or late."
        )
        return result
    low, high = values[at], values[at + 1]
    result["recommendation"] = round((low + high) / 2, 3)
    result["note"] = (
        f"The jumped hits split into {at + 1} with {low:.3f} or less and {len(values) - at - 1} "
        f"with {high:.3f} or more; a threshold between them is {result['recommendation']}."
    )
    return result


def settings_path(argv):
    if argv:
        return Path(argv[0])
    mods = os.environ.get("CINNABAR_MODS_DIR")
    if not mods:
        local = os.environ.get("LOCALAPPDATA")
        mods = str(Path(local) / "Cinnabar" / "mods") if local else ""
    return Path(mods) / "block-outline.settings.json"


def report(path):
    try:
        settings = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        print(f"Could not read {path}: {error}", file=sys.stderr)
        return 2
    rows = parse_log(settings.get("jump_reset_log", ""))
    if not rows:
        print("The settings file has no recorded hits yet. Take some hits with Jump reset on.")
        return 1
    window = settings.get("jump_reset_window", DEFAULT_WINDOW_MS)
    threshold = settings.get("jump_reset_carry", DEFAULT_CARRY)
    result = analyse(rows, window, threshold)
    print(f"{result['hits']} hits recorded, {result['grounded']} on the ground "
          f"({result['jumped']} with a jump inside {window * WINDOW_FACTOR} ms, "
          f"{result['other']} without).")
    print(f"Current threshold {threshold}: {result['counted']} of {result['jumped']} jumped hits "
          "count as a reset.")
    print(result["note"])
    if result["recommendation"] is not None:
        print(f'Suggested: "jump_reset_carry": {result["recommendation"]}')
    return 0


def self_test():
    rows = parse_log("12,40,1,300;-30,150,1,310;x,210,1,300;5,35,1,290;100,180,0,330;bad;3,2")
    assert rows == [
        (12, 0.04, True, 0.3),
        (-30, 0.15, True, 0.31),
        (None, 0.21, True, 0.3),
        (5, 0.035, True, 0.29),
        (100, 0.18, False, 0.33),
    ], rows

    few = analyse(rows, 75, 0.08)
    assert few["recommendation"] is None and "Not enough data" in few["note"]

    # Resets near 0.04, failed jumps near 0.17, plus hits with no jump.
    resets = [(0, 0.03 + 0.005 * i, True, 0.3) for i in range(4)]
    failed = [(-60 + 5 * i, 0.15 + 0.01 * i, True, 0.3) for i in range(4)]
    plain = [(None, 0.2, True, 0.3) for _ in range(4)]
    clean = analyse(resets + failed + plain, 75, 0.08)
    assert clean["jumped"] == 8 and clean["counted"] == 4, clean
    assert clean["recommendation"] is not None and 0.045 < clean["recommendation"] < 0.15, clean

    # Hits taken in the air are ignored.
    air = [(0, 0.3, False, 0.3) for _ in range(20)]
    assert analyse(resets + failed + plain + air, 75, 0.08)["grounded"] == 12

    # Every jump the same: no split to find.
    same = [(0, 0.1, True, 0.3) for _ in range(12)]
    assert "No clean split" in analyse(same, 75, 0.08)["note"]
    print("jump-reset-report self-test passed")
    return 0


def main():
    args = sys.argv[1:]
    if args == ["--self-test"]:
        return self_test()
    return report(settings_path(args))


if __name__ == "__main__":
    sys.exit(main())
