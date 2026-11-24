#!/usr/bin/env python3
"""Write KernelSU update metadata as UTF-8 bytes on every host platform."""

import argparse
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release-json", required=True, type=Path)
    parser.add_argument("--version-code", required=True, type=int)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    release = json.loads(args.release_json.read_text(encoding="utf-8"))
    version = release["tag_name"]
    notes = release.get("body") or ""
    if not isinstance(version, str) or not version or not isinstance(notes, str):
        raise ValueError("Release tag and notes must be strings")
    if args.version_code <= 0:
        raise ValueError("Version code must be positive")
    assets = [
        asset for asset in release["assets"]
        if "release" in asset["name"] and asset["name"].endswith(".zip")
        and asset.get("state") == "uploaded"
    ]
    # arm64 keeps the unsuffixed package and the canonical update.json; x86_64
    # uses a separate ABI-marked package and update file so each device updates
    # to its own architecture.
    arm64 = [asset for asset in assets if "-x86_64-" not in asset["name"]]
    x86_64 = [asset for asset in assets if "-x86_64-" in asset["name"]]
    if len(arm64) != 1 or len(x86_64) > 1:
        raise ValueError(
            f"Expected one arm64 and at most one x86_64 release ZIP, got "
            f"{len(arm64)} and {len(x86_64)}: {[a['name'] for a in assets]}"
        )
    common = {
        "version": version,
        "versionCode": args.version_code,
        "changelog": "https://raw.githubusercontent.com/ITxiao6666/OhMyKeymint/changelog/changelog.md",
    }
    updates = {
        "update.json": {**common, "zipUrl": arm64[0]["browser_download_url"]},
    }
    # Older releases carry only the arm64 package; keep their update.json valid.
    if x86_64:
        updates["update-x86_64.json"] = {**common, "zipUrl": x86_64[0]["browser_download_url"]}
    # Explicit bytes avoid Windows locale encoding and newline conversion. Never
    # pipe non-ASCII metadata through subprocess text mode without UTF-8 encoding.
    normalized_notes = notes.replace("\r\n", "\n").replace("\r", "\n").rstrip("\n") + "\n"
    encoded_notes = normalized_notes.encode("utf-8")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    (args.output_dir / "changelog.md").write_bytes(encoded_notes)
    for name, update in updates.items():
        encoded_update = (json.dumps(update, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
        (args.output_dir / name).write_bytes(encoded_update)


if __name__ == "__main__":
    main()
