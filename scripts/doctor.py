#!/usr/bin/env python3
"""Says why Boldy Mods is (or is not) working with the Cinnabar installed right now.

Cinnabar updates itself (the Nightly channel, and "Install updates when I quit"), while the mod is
built against one fixed `mod-api` revision. When the mod misbehaves, run this first: it only reads,
changes nothing, and says which of the usual causes it is.

    python scripts/doctor.py            # diagnose
    python scripts/doctor.py --record   # remember this host as known good (do it once the mod
                                        # works in game); later runs say what changed since
    python scripts/doctor.py --offline  # skip the GitHub lookups
    python scripts/doctor.py --self-test

It checks: the state Cinnabar reports for the mod, the permission grants Cinnabar stored, whether
the installed file is the one in `dist/`, whether the host still supports the API revision the mod
was built for, which host changes touched `crates/mod-api` since the known-good host, and the
recent logs for a quarantine. `CINNABAR_DIR` / `CINNABAR_MODS_DIR` override the folders.
Exit status 1 means at least one FAIL.
"""

from pathlib import Path
import datetime
import hashlib
import json
import os
import re
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
INSTALL_NAME = "block-outline.wasm"
KNOWN_GOOD = ROOT / "docs" / "known-good-host.json"
CHANNELS = ["Beta", "Nightly", "Stable"]  # the order of Cinnabar's `update_channel` setting
REPOSITORY = "cinnabar-mc/cinnabar"
TROUBLE = re.compile(r"quarantin|all fuel consumed|wasm trap|block-outline|boldy", re.IGNORECASE)
# The packet mod (Hive Auto CS) writes a line per packet; it is not what this tool looks at.
NOISE = re.compile(r"autocs\]|packet_mod")


# --- pure helpers (covered by --self-test) -------------------------------------------------------


def parse_core_build(text):
    """The (version, commit) of the newest `core build` line in Cinnabar's core log."""
    found = None
    for line in text.splitlines():
        if 'msg="core build"' in line:
            version = re.search(r"\bversion=(\S+)", line)
            commit = re.search(r"\bcommit=([0-9a-f]{7,40})", line)
            if version and commit:
                found = (version.group(1), commit.group(1))
    return found


def mod_state(status, name):
    """(state, message) Cinnabar reports for the component called `name`, or None."""
    for entry in status.get("mods", []):
        if str(entry.get("component", "")).lower().endswith(name.lower()):
            return entry.get("state"), entry.get("message")
    return None


def mod_entry(registry, name):
    """The registration (enabled flag and grants) Cinnabar stored for `name`, or None."""
    for entry in registry.get("mods", []):
        if str(entry.get("component", "")).lower().endswith(name.lower()):
            return entry
    return None


def missing_grants(wanted, granted):
    """The permissions the mod asks for that Cinnabar's stored grants leave off."""
    missing = []
    for key, on in wanted.items():
        have = granted.get(key)
        if isinstance(on, list):
            # A list grant (the commands) names what it needs; say which names are left off.
            lacking = [name for name in on if name not in (have or [])]
            if lacking:
                missing.append(f"{key} ({', '.join(lacking)})")
        elif on and not have:
            missing.append(key)
    return sorted(missing)


def pinned_revision(cargo_toml_text):
    """The `mod-api` or `ui` git revision pinned in a Cargo.toml, or None."""
    match = re.search(r'^(?:mod-api|ui)\s*=\s*\{[^}]*rev\s*=\s*"([0-9a-f]{40})"', cargo_toml_text, re.M)
    return match.group(1) if match else None


def newest_revision(revisions):
    """The API revision a source tree was built for: the last entry of its list, oldest first."""
    return revisions[-1] if revisions else None


def trouble_lines(lines):
    """Log lines that mention the mod or a quarantine, without the packet mod's chatter."""
    return [line.strip() for line in lines if TROUBLE.search(line) and not NOISE.search(line)]


# --- reading the machine -------------------------------------------------------------------------


def cinnabar_dir():
    configured = os.environ.get("CINNABAR_DIR")
    local = os.environ.get("LOCALAPPDATA")
    return Path(configured) if configured else Path(local) / "Cinnabar" if local else None


def read_json(path):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def sha256(path):
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()
    except OSError:
        return None


def fetch_json(url):
    request = urllib.request.Request(url, headers={"User-Agent": "boldy-mods-doctor"})
    try:
        with urllib.request.urlopen(request, timeout=8) as reply:
            return json.load(reply)
    except (OSError, ValueError):
        return None


def revisions_at(commit, offline):
    """`compatibility-revisions.json` at a Cinnabar commit: a local cargo checkout, else GitHub."""
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    for checkout in cargo_home.glob(f"git/checkouts/cinnabar-*/{commit[:7]}*"):
        found = read_json(checkout / "crates/mod-api/compatibility-revisions.json")
        if found is not None:
            return found
    if offline:
        return None
    url = f"https://raw.githubusercontent.com/{REPOSITORY}/{commit}/crates/mod-api/compatibility-revisions.json"
    return fetch_json(url)


def mod_api_changes(since_commit, until_commit):
    """Host commits that touched `crates/mod-api` after `since_commit`, newest first, or None."""
    api = f"https://api.github.com/repos/{REPOSITORY}"
    since = fetch_json(f"{api}/commits/{since_commit}")
    if not since:
        return None
    date = since["commit"]["committer"]["date"]
    listed = fetch_json(f"{api}/commits?sha={until_commit}&path=crates/mod-api&since={date}&per_page=30")
    if listed is None:
        return None
    return [
        f"{item['sha'][:9]} {item['commit']['message'].splitlines()[0]}"
        for item in listed
        if not item["sha"].startswith(since_commit[:9])
    ]


# --- the checks ----------------------------------------------------------------------------------


class Report:
    def __init__(self):
        self.failed = False

    def line(self, level, title, detail=""):
        if level == "FAIL":
            self.failed = True
        print(f"[{level:^4}] {title}")
        for extra in detail.splitlines():
            print(f"         {extra}")


def check(offline):
    report = Report()
    home = cinnabar_dir()
    mods = Path(os.environ["CINNABAR_MODS_DIR"]) if os.environ.get("CINNABAR_MODS_DIR") else home / "mods" if home else None
    if home is None or not home.is_dir():
        report.line("SKIP", "Cinnabar's data folder was not found", "Set CINNABAR_DIR to %LOCALAPPDATA%\\Cinnabar.")
        return report, None

    # The host: which build is running, and whether it keeps changing under the mod.
    core = (home / "logs" / "core.log")
    build = parse_core_build(core.read_text(encoding="utf-8", errors="replace")) if core.exists() else None
    settings = read_json(home / "settings.json") or {}
    values = settings.get("values", {})
    channel = values.get("update_channel", 0)
    channel_name = CHANNELS[channel] if isinstance(channel, int) and 0 <= channel < len(CHANNELS) else str(channel)
    label = f"Cinnabar {build[0]} (core {build[1][:9]})" if build else "Cinnabar (version not found in the core log)"
    report.line("INFO", f"Host: {label}, update channel {channel_name}")
    if channel_name != "Beta" or values.get("install_updates_on_quit"):
        report.line(
            "WARN",
            "Cinnabar updates itself often",
            "Nightly builds and Install-updates-on-quit swap the host between sessions.\n"
            "Settings > Updates: choose Beta and turn off 'Install updates when I quit', then update\n"
            "on purpose and run this script afterwards. See docs/stability.md.",
        )

    # The mod: the state Cinnabar reports.
    status = read_json(home / "local-mod.status.json") or {}
    state = mod_state(status, INSTALL_NAME)
    if state is None:
        report.line("FAIL", "Cinnabar does not list Boldy Mods", "Import dist\\block-outline.wasm in Settings > Mods, or run build-mod.py.")
    elif state[0] == "loaded":
        report.line("OK", "Cinnabar reports the mod as loaded")
    elif state[0] == "next-join":
        report.line("WARN", "The mod starts on your next join", state[1] or "")
    else:
        report.line("FAIL", f"Cinnabar reports the mod as '{state[0]}'", (state[1] or "") + "\nRead the log lines below, then the 'Quarantine' section of docs/stability.md.")

    # The grants: a missing permission silently switches a feature off.
    registry = read_json(home / "local-mod.json") or {}
    entry = mod_entry(registry, INSTALL_NAME)
    wanted = read_json(ROOT / "mods/boldy-mods/pack/grants.json") or {}
    if entry is None:
        report.line("WARN", "Cinnabar has no stored registration or grants for the mod")
    else:
        if not entry.get("enabled", True):
            report.line("FAIL", "The mod is switched off in Settings > Mods")
        missing = missing_grants(wanted, entry.get("grants", {}))
        if missing:
            report.line("FAIL", "Permissions the mod needs are not granted", ", ".join(missing) + "\nAllow them in Settings > Mods (a host update can reset grants).")
        else:
            report.line("OK", "Every permission the mod asks for is granted")

    # The file: is the installed copy the one that was built here?
    installed = (mods / INSTALL_NAME) if mods else None
    built = ROOT / "dist" / INSTALL_NAME
    if installed is None or not installed.exists():
        report.line("FAIL", f"{INSTALL_NAME} is not in the mods folder", "python scripts\\build-mod.py")
    elif built.exists() and sha256(installed) != sha256(built):
        report.line("WARN", "The installed file differs from dist\\block-outline.wasm", "Something else replaced it, or dist is stale: run python scripts\\build-mod.py.")
    else:
        report.line("OK", "The installed file is the one in dist\\" if built.exists() else "The mod file is installed")
    if mods and (mods / "block-outline.settings.json").exists() and read_json(mods / "block-outline.settings.json") is None:
        report.line("FAIL", "block-outline.settings.json is not valid JSON", "The mod falls back to defaults; fix or delete the file.")

    # The API: does the host still run what this mod was built against?
    pin = pinned_revision((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    host_pin = pinned_revision((ROOT / "tools/host-check/Cargo.toml").read_text(encoding="utf-8"))
    if pin and host_pin and pin != host_pin:
        report.line("FAIL", "mod-api and tools/host-check pin different Cinnabar revisions", f"{pin[:9]} vs {host_pin[:9]}: change both together.")
    built_for = newest_revision(revisions_at(pin, offline) or []) if pin else None
    host_list = revisions_at(build[1], offline) if build and len(build[1]) >= 7 else None
    if not built_for or host_list is None:
        report.line("SKIP", "API revision check", "Needs the pinned and the installed Cinnabar source (offline, or GitHub unreachable).")
    elif built_for in host_list:
        extra = len(host_list) - host_list.index(built_for) - 1
        report.line("OK", f"The host still supports the mod's API revision ({built_for[:9]})", f"The host has {extra} newer revision(s); the mod does not need them." if extra else "")
    else:
        report.line("FAIL", f"The host no longer supports the mod's API revision ({built_for[:9]})", "Bump the mod-api pin: follow 'Updating the pin' in docs/stability.md.")

    # What changed in the host since it last worked.
    known = read_json(KNOWN_GOOD)
    if not known:
        report.line("INFO", "No known-good host recorded", "When the mod works in game, run: python scripts\\doctor.py --record")
    elif build and known.get("cinnabar_commit", "").startswith(build[1][:9]):
        report.line("OK", f"The host is the known-good one ({known.get('cinnabar_version')}, recorded {known.get('recorded')})")
    elif build:
        changes = None if offline else mod_api_changes(known["cinnabar_commit"], build[1])
        head = f"The host changed since the known-good {known.get('cinnabar_version')} ({known.get('cinnabar_commit', '')[:9]}, {known.get('recorded')})"
        if changes is None:
            report.line("WARN", head, "Could not list the changes to crates/mod-api (offline or GitHub unreachable).")
        elif changes:
            report.line("WARN", head, f"{len(changes)} commit(s) touched crates/mod-api; these can change behaviour:\n" + "\n".join(changes[:12]))
        else:
            report.line("INFO", head, "No commit touched crates/mod-api: only game or renderer changes.")

    # The logs: a quarantine or a trap names itself there.
    found = []
    logs = sorted((home / "logs").glob("*.log"), key=lambda p: p.stat().st_mtime, reverse=True)[:8]
    for path in logs:
        try:
            with path.open(encoding="utf-8", errors="replace") as handle:
                found += [f"{path.name}: {line[:200]}" for line in trouble_lines(handle)]
        except OSError:
            pass
    if found:
        report.line("WARN", f"{len(found)} log line(s) mention the mod or a quarantine (newest logs first)", "\n".join(found[:6]))
    else:
        report.line("OK", "The latest logs show no quarantine, trap or mod error")
    return report, build


def record(build):
    if not build:
        print("Cannot record: the Cinnabar version was not found in its core log.")
        return 1
    pin = pinned_revision((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    document = {
        "cinnabar_version": build[0],
        "cinnabar_commit": build[1],
        "mod_api_pin": pin,
        "recorded": datetime.date.today().isoformat(),
    }
    KNOWN_GOOD.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    print(f"Recorded Cinnabar {build[0]} ({build[1][:9]}) as known good in {KNOWN_GOOD.relative_to(ROOT)}.")
    return 0


# --- self test -----------------------------------------------------------------------------------


def self_test():
    log = 'time=1 level=INFO msg="core build" process=a version=0.2.1 commit=638898362e7888d3f4cf918fc078ff8a416a91f7 go=1\n'
    assert parse_core_build(log) == ("0.2.1", "638898362e7888d3f4cf918fc078ff8a416a91f7")
    assert parse_core_build("nothing here") is None
    newer = log.replace("0.2.1", "0.2.2").replace("638898362e", "aaaaaaaaaa")
    assert parse_core_build(log + newer)[0] == "0.2.2", "the newest build line wins"

    status = {"mods": [{"component": "\\\\?\\C:\\m\\hive-autocs.wasm", "state": "next-join", "message": "x"},
                       {"component": "\\\\?\\C:\\m\\block-outline.wasm", "state": "loaded", "message": None}]}
    assert mod_state(status, "block-outline.wasm") == ("loaded", None)
    assert mod_state(status, "missing.wasm") is None
    assert mod_entry({"mods": [{"component": "C:\\block-outline.wasm", "enabled": False}]}, INSTALL_NAME)["enabled"] is False

    assert missing_grants({"hud": True, "camera": True, "audio": False}, {"hud": True}) == ["camera"]
    assert missing_grants({"hud": True}, {"hud": True, "audio": True}) == []
    assert missing_grants({"commands": ["q", "rq"]}, {"commands": []}) == ["commands (q, rq)"]
    assert missing_grants({"commands": ["q", "rq", "p"]}, {"commands": ["q", "rq"]}) == ["commands (p)"]
    assert missing_grants({"commands": ["q", "rq"]}, {"commands": ["rq", "q", "say"]}) == []

    assert pinned_revision('mod-api = { git = "g", rev = "' + "a" * 40 + '" }') == "a" * 40
    assert pinned_revision('ui = { git = "g", rev = "' + "b" * 40 + '" }\nserde_json = "1"') == "b" * 40
    assert pinned_revision("nothing") is None
    assert newest_revision(["old", "new"]) == "new" and newest_revision([]) is None

    quiet = trouble_lines(["[autocs] S>C packet", "ordinary line", "wasm trap: all fuel consumed", "block-outline quarantined"])
    assert quiet == ["wasm trap: all fuel consumed", "block-outline quarantined"], quiet

    # The two Cinnabar pins in the repository must never drift apart.
    pins = {pinned_revision((ROOT / path).read_text(encoding="utf-8")) for path in ("Cargo.toml", "tools/host-check/Cargo.toml")}
    assert len(pins) == 1 and None not in pins, f"mod-api and host-check pin different revisions: {pins}"
    print("doctor self-test passed")


def main():
    arguments = sys.argv[1:]
    if "--self-test" in arguments:
        self_test()
        return 0
    print("Boldy Mods doctor (read-only)")
    report, build = check("--offline" in arguments)
    if "--record" in arguments:
        if report.failed:
            print("\nNot recording: fix the FAIL lines above first, then check the mod in game.")
            return 1
        return record(build)
    print("\n" + ("Something needs fixing: see the FAIL lines." if report.failed else "No failure found by this tool. If it still misbehaves, say what you see and run with the newest logs in hand."))
    return 1 if report.failed else 0


if __name__ == "__main__":
    sys.exit(main())
