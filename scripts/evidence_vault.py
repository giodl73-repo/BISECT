#!/usr/bin/env python3
"""Restore hash-bound historical evidence from a vault or pinned Git history."""
import argparse
import json
import subprocess
import tempfile
from pathlib import Path

if __package__ in (None, ""):
    import sys
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.data_vault import ROOT, inside, load_versioned, sha256, copy_checked, vault_root, valid_hash

MANIFEST = ROOT / "configs/data-vault/evidence-v1.json"


def entries():
    rows = load_versioned(MANIFEST, "bisect-evidence-catalog-v1")["items"]
    seen = set()
    for row in rows:
        inside(ROOT, row["repo_path"])
        inside(ROOT, row["vault_path"])
        if row["repo_path"] in seen or not valid_hash(row["sha256"]) or row["bytes"] <= 0:
            raise ValueError("invalid evidence entry")
        if len(row["revision"]) != 40 or any(c not in "0123456789abcdef" for c in row["revision"]):
            raise ValueError("invalid historical revision")
        seen.add(row["repo_path"])
    return rows


def hydrate(vault, repo=ROOT, apply=False):
    results = []
    for row in entries():
        target = inside(repo, row["repo_path"])
        source = inside(vault, row["vault_path"])
        if target.exists():
            if target.stat().st_size != row["bytes"] or sha256(target) != row["sha256"]:
                raise ValueError(f"existing evidence differs; not overwritten: {target}")
            results.append({"path": row["repo_path"], "status": "verified"})
            continue
        if not apply:
            results.append({"path": row["repo_path"], "status": "would-hydrate", "source": "vault" if source.exists() else "pinned-git"})
            continue
        if not source.exists():
            vault.mkdir(parents=True, exist_ok=True)
            with tempfile.TemporaryDirectory(prefix="bisect-history-", dir=vault) as temp:
                staged = Path(temp) / "evidence"
                with staged.open("xb") as out:
                    subprocess.run(["git", "show", row["revision"] + ":" + row["repo_path"]], cwd=ROOT, stdout=out, check=True, timeout=600)
                copy_checked(staged, source, row["sha256"], row["bytes"])
        copy_checked(source, target, row["sha256"], row["bytes"])
        results.append({"path": row["repo_path"], "status": "hydrated-verified"})
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vault")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--check-size", action="store_true", help="check Git's indexed snapshot against the 650 MB budget")
    args = parser.parse_args()
    if args.check_size:
        if args.vault or args.apply:
            parser.error("--check-size does not take --vault/--apply")
        listing = subprocess.check_output(["git", "ls-files", "--stage", "-z"], cwd=ROOT).split(b"\0")
        objects = [line.split(b" ")[1] for line in listing if line]
        sizes = subprocess.check_output(["git", "cat-file", "--batch-check=%(objectsize)"], input=b"\n".join(objects) + b"\n", cwd=ROOT)
        total = sum(int(size) for size in sizes.splitlines())
        print(json.dumps({"indexed_bytes": total, "budget_bytes": 650_000_000}))
        return int(total > 650_000_000)
    print(json.dumps(hydrate(vault_root(args.vault), apply=args.apply), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
