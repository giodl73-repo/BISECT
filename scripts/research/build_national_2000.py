#!/usr/bin/env python3
"""Download governed Census 2000 county geometry and build state RCTX packages."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.request
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / "docs/experiments/nationwide-2000/inventory.json"
COMPONENT_EXTENSIONS = {".shp", ".dbf", ".shx"}


def download(url: str, destination: Path) -> None:
    if destination.is_file():
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    partial = destination.with_suffix(destination.suffix + ".part")
    for attempt in range(1, 5):
        try:
            request = urllib.request.Request(url, headers={"User-Agent": "BISECT-NRS-custody/0.2"})
            with urllib.request.urlopen(request, timeout=120) as response, partial.open("wb") as target:
                expected = response.headers.get("Content-Length")
                copied = 0
                while chunk := response.read(8 * 1024 * 1024):
                    target.write(chunk)
                    copied += len(chunk)
            if expected is not None and copied != int(expected):
                raise OSError(f"short download: expected {expected} bytes, received {copied}")
            os.replace(partial, destination)
            return
        except Exception:
            if partial.exists():
                partial.unlink()
            if attempt == 4:
                raise
            time.sleep(2**attempt)


def extract_components(archive: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive) as source:
        members = [
            member
            for member in source.infolist()
            if Path(member.filename).suffix.lower() in COMPONENT_EXTENSIONS
        ]
        archive_stem = archive.stem.lower()
        member_stems = {Path(member.filename).stem.lower() for member in members}
        if (
            len(members) != 3
            or {Path(member.filename).suffix.lower() for member in members} != COMPONENT_EXTENSIONS
            or member_stems != {archive_stem}
        ):
            raise RuntimeError(f"archive lacks required shapefile components: {archive}")
        for member in members:
            name = Path(member.filename).name
            if not name or name != member.filename.replace("\\", "/").split("/")[-1]:
                raise RuntimeError(f"unsafe TIGER member path: {member.filename}")
            target = destination / name
            with source.open(member) as incoming, target.open("wb") as outgoing:
                shutil.copyfileobj(incoming, outgoing, length=8 * 1024 * 1024)


def build_state(binary: Path, row: dict[str, object]) -> None:
    code = str(row["state"])
    lower = code.lower()
    archive_dir = ROOT / f"data/2000/tiger/archives/{lower}"
    block_dir = ROOT / f"data/2000/tiger/blocks/{lower}"
    report_dir = ROOT / "docs/experiments/nationwide-2000/rctx"
    report_dir.mkdir(parents=True, exist_ok=True)
    certified = ROOT / "data/2000/certified"
    certified.mkdir(parents=True, exist_ok=True)
    geography = ROOT / f"data/2000/redistricting/{lower}geo.upl"
    command = [
        str(binary), "build-state-rctx", "--year=2000",
        f"--state-code={code}", f"--state-fips={row['fips']}",
        f"--state-name={str(row['name']).lower().replace(' ', '_')}",
        f"--shapefile={block_dir}", f"--tiger-archive={archive_dir}",
        f"--pl-geo={geography}", f"--pl-population={geography}",
        f"--rctx={certified / f'{lower}_blocks_2000.rctx'}",
        f"--report={report_dir / f'{lower}.json'}",
        f"--manifest={report_dir / f'{lower}-manifest.json'}",
    ]
    subprocess.run(command, cwd=ROOT, check=True)


def clean_extracted_state(block_dir: Path) -> None:
    governed_root = (ROOT / "data/2000/tiger/blocks").resolve()
    resolved = block_dir.resolve()
    if resolved.parent != governed_root or not resolved.is_dir():
        raise RuntimeError(f"refusing to remove ungoverned extraction path: {resolved}")
    shutil.rmtree(resolved)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", action="append", default=[], help="two-letter state filter")
    parser.add_argument("--download-only", action="store_true")
    parser.add_argument("--keep-extracted", action="store_true")
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/bisect-ops.exe")
    args = parser.parse_args()

    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    by_state = {row["state"]: row for row in inventory["states"]}
    requested = {state.upper() for state in args.state}
    unknown = requested - set(by_state)
    if unknown:
        parser.error(f"unknown states: {','.join(sorted(unknown))}")
    order = [state for state in inventory["batch_order"] if not requested or state in requested]
    if not args.download_only and not args.binary.is_file():
        subprocess.run(["cargo", "build", "--release", "-p", "bisect-ops"], cwd=ROOT, check=True)

    for position, code in enumerate(order, 1):
        row = by_state[code]
        lower = code.lower()
        archive_dir = ROOT / f"data/2000/tiger/archives/{lower}"
        block_dir = ROOT / f"data/2000/tiger/blocks/{lower}"
        print(
            f"[{position}/{len(order)}] {code}: {row['county_count']} county archives, "
            f"{row['block_count']} blocks",
            flush=True,
        )
        for url in row["tiger_source_urls"]:
            archive = archive_dir / Path(url).name
            download(url, archive)
            extract_components(archive, block_dir)
        if args.download_only:
            continue
        build_state(args.binary, row)
        if not args.keep_extracted:
            clean_extracted_state(block_dir)
            print(f"{code}: removed reproducible extracted TIGER components", flush=True)

    subprocess.run([sys.executable, str(ROOT / "scripts/research/inventory_national_2000.py")], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
