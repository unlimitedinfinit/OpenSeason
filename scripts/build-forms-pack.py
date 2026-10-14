#!/usr/bin/env python3
"""Build a forms-pack zip and catalog snippet from a folder of files.

Does not download court PDFs. Hashes whatever is already in the folder.
Official forms must stay out of git.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import zipfile
from pathlib import Path


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def iter_files(root: Path):
    for path in sorted(root.rglob("*")):
        if path.is_file():
            yield path


def main() -> int:
    parser = argparse.ArgumentParser(description="Build an OpenSeason forms pack archive.")
    parser.add_argument("folder", type=Path, help="Pack directory (contains pack.json and form files)")
    parser.add_argument("--out", type=Path, required=True, help="Destination .zip")
    args = parser.parse_args()
    folder = args.folder.resolve()
    if not folder.is_dir():
        print(f"Not a directory: {folder}", file=sys.stderr)
        return 1
    pack_json = folder / "pack.json"
    pack = {}
    if pack_json.is_file():
        pack = json.loads(pack_json.read_text())
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.out, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for path in iter_files(folder):
            zf.write(path, path.relative_to(folder).as_posix())
    digest = sha256_file(args.out)
    size = args.out.stat().st_size
    forms = []
    for form in pack.get("forms", []):
        rel = form.get("path_in_pack") or ""
        file_path = folder / rel if rel else None
        entry = {
            "id": form.get("id"),
            "state": form.get("state") or pack.get("jurisdiction"),
            "court": form.get("court") or pack.get("court"),
            "situation": form.get("situation"),
            "title": form.get("title"),
            "size_bytes": file_path.stat().st_size if file_path and file_path.is_file() else 0,
            "download_url": form.get("download_url") or "https://example.test/forms/REPLACE",
            "sha256": sha256_file(file_path) if file_path and file_path.is_file() else "",
        }
        forms.append(entry)
    snippet = {
        "id": pack.get("id") or args.out.stem,
        "state": pack.get("jurisdiction", ""),
        "court": pack.get("court", ""),
        "situations": [s.get("id") for s in pack.get("situations", [])],
        "title": pack.get("name") or args.out.stem,
        "size_bytes": size,
        "download_url": f"https://example.test/packs/{args.out.name}",
        "sha256": digest,
        "rule_pack_id": pack.get("rule_pack_id") or pack.get("id"),
        "source_note": "Replace download_url with your GitHub Release, R2, or HTTPS URL.",
    }
    print(json.dumps({"pack": snippet, "forms": forms}, indent=2))
    print(f"Wrote {args.out} ({size} bytes) sha256={digest}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
