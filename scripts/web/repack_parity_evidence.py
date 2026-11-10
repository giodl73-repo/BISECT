"""Rebalance an evidence set without changing or running any retained files."""

import argparse
import hashlib
import os
from pathlib import Path

import parity_evidence_archive as archive


def index(parts):
    rows = {}
    for _, manifest in parts:
        for row in manifest["files"]:
            name = row["path"]
            archive.require(name not in rows, "Duplicate evidence destination")
            rows[name] = (row["sha256"], row["bytes"])
    return rows


def bins_for(rows, limit):
    bins = []
    for name, (sha256, size) in sorted(rows.items(), key=lambda item: (-item[1][1], item[0])):
        archive.require(size <= limit, "An evidence file exceeds the target shard size")
        for bin_ in bins:
            if bin_["bytes"] + size <= limit and len(bin_["transfer"]) < archive.MAX_FILES:
                break
        else:
            bin_ = {"bytes": 0, "transfer": []}
            bins.append(bin_)
        bin_["bytes"] += size
        bin_["transfer"].append({"destination": name, "sha256": sha256, "byte_length": size})
    # Keep a set even if all bytes can fit in one shard.
    if len(bins) == 1:
        archive.require(len(rows) > 1, "A one-file archive does not need rebalancing")
        moved = bins[0]["transfer"].pop()
        bins[0]["bytes"] -= moved["byte_length"]
        bins.append({"bytes": moved["byte_length"], "transfer": [moved]})
    archive.require(len(bins) <= archive.MAX_SHARDS, "Rebalanced set exceeds the shard cap")
    return bins


def repack(root, old_dir, old_lock, output, limit, merge_shards=()):
    archive.require(root.is_dir() and not output.exists(), "Use an existing restored root and a fresh output")
    archive.require(0 < limit <= archive.MAX_TOTAL, "Invalid decoded shard size")
    kept = []
    with archive.verified_capsule(old_dir, old_lock) as (parts, old):
        original = index(parts)
        if merge_shards:
            trusted_bytes, trusted = archive.read_lock(old_lock)
            archive.require(hashlib.sha256(trusted_bytes).hexdigest() == old.get("set_sha256"),
                            "Partial repack requires the unchanged verified set lock")
            requested = set(merge_shards)
            names = {row['name'] for row in trusted['shards']}
            archive.require(len(requested) == len(merge_shards) and 2 <= len(requested) < len(names)
                            and requested <= names, "Select at least two distinct existing shards; leave one unchanged")
            selected = []
            for row, part in zip(trusted['shards'], parts, strict=True):
                if row['name'] in requested:
                    selected.append(part)
                else:
                    kept.append(row)
            inputs = index(selected)
            # Unlike a complete repack, one replacement shard is useful here.
            archive.require(sum(size for _, size in inputs.values()) <= limit and len(inputs) <= archive.MAX_FILES,
                            "Selected evidence does not fit into one shard")
            bins = [{"bytes": sum(size for _, size in inputs.values()), "transfer": [
                {"destination": name, "sha256": sha, "byte_length": size}
                for name, (sha, size) in sorted(inputs.items())]}]
        else:
            bins = bins_for(original, limit)
    archive.require(len(bins) + len(kept) < old.get("shards", 1), "Repacking would not reduce the number of shards")
    archive.require(not kept or 'consolidated.zip' not in {r['name'] for r in kept},
                    "Include the existing consolidated shard in the merge")
    output.mkdir()
    arguments = []
    for row in kept:
        name = row['name']
        # Keep the verified archive bytes without copying large shards. A
        # cross-volume request fails rather than allocating an unplanned copy.
        os.link(old_dir / name, output / name)
        lock = output / (name + '.lock.json')
        lock.write_bytes(archive.canonical(row['lock']))
        arguments.append(f"{name}={lock}")
    for ordinal, bin_ in enumerate(bins, 1):
        name = 'consolidated' if kept else f"evidence-{ordinal:02d}"
        transfer, lock = output / (name + ".transfer.json"), output / (name + ".lock.json")
        transfer.write_bytes(archive.canonical({"transfer": bin_["transfer"]}))
        archive.export_archive(root, transfer, output / (name + ".zip"), lock)
        arguments.append(f"{name}.zip={lock}")
    new_lock = output / "set.lock.json"
    archive.compose_set(output, arguments, new_lock)
    with archive.verified_capsule(output, new_lock) as (parts, new):
        archive.require(index(parts) == original, "Repacked destination, digest or byte length changed")
    archive.require(new["bytes"] == old["bytes"] and new["files"] == old["files"],
                    "Repacked set totals changed")
    identity = hashlib.sha256(archive.canonical(original)).hexdigest()
    result = {"status": "passed-byte-preserving-evidence-repack", "old": old, "new": new,
              "logical_index_sha256": identity, "decoded_shard_bytes": [b["bytes"] for b in bins],
              "executed_files": 0}
    if kept:
        result['unchanged_shards'] = [row['name'] for row in kept]
        result['merged_shards'] = sorted(merge_shards)
    (output / "receipt.json").write_bytes(archive.canonical(result))
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("root", "old-dir", "old-lock", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--max-shard-mib", type=int, default=224)
    parser.add_argument("--merge-shard", action="append", default=[],
                        help="Merge only these named ZIPs; hard-link unchanged shards on the same volume")
    args = parser.parse_args()
    result = repack(args.root, args.old_dir, args.old_lock, args.output, args.max_shard_mib * 1024**2, args.merge_shard)
    print(archive.canonical(result).decode(), end="")
