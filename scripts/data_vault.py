#!/usr/bin/env python3
"""Portable, opt-in data-vault setup. Standard library only; no implicit M: path."""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
import os
import platform
import re
import stat
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "configs/data-vault/catalog-v1.json"
LOCAL = ROOT / ".bisect-vault.json"


def load_versioned(path, schema):
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or data.get("schema") != schema:
        raise ValueError(f"unsupported schema: {path}")
    return data


def valid_hash(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def load_catalog():
    data = load_versioned(CATALOG, "bisect-data-catalog-v1")
    rows = data.get("items")
    if not isinstance(rows, list) or not rows:
        raise ValueError("empty or invalid catalog")
    ids, paths = set(), set()
    for row in rows:
        if not isinstance(row, dict) or not {"id", "path", "bytes", "sha256", "state", "year", "group"} <= row.keys():
            raise ValueError("missing catalog fields")
        inside(ROOT, row["path"])
        if (not valid_hash(row["sha256"]) or type(row["bytes"]) is not int or row["bytes"] <= 0
                or row["year"] not in (2000, 2010, 2020)
                or row["group"] not in ("contexts", "blocks", "population", "enacted")
                or not isinstance(row["state"], str) or not re.fullmatch(r"[A-Z]{2}", row["state"])):
            raise ValueError("invalid catalog fields")
        if row["id"] in ids or row["path"].casefold() in paths:
            raise ValueError("duplicate catalog id/path")
        ids.add(row["id"])
        paths.add(row["path"].casefold())
        if "url" in row:
            census_url(row["url"])
        if "member" in row and (not isinstance(row["member"], str) or PurePosixPath(row["member"]).name != row["member"]):
            raise ValueError("invalid archive member")
    states = set("AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY".split())
    for year in (2000, 2010, 2020):
        contexts = [r for r in rows if r["year"] == year and r["group"] == "contexts"]
        if len(contexts) != 50 or {r["state"] for r in contexts} != states:
            raise ValueError("incomplete context State coverage")
    for group, count in (("blocks", 1), ("population", 2), ("enacted", 1)):
        for state in states:
            if sum(r["year"] == 2020 and r["state"] == state and r["group"] == group for r in rows) != count:
                raise ValueError("incomplete source State coverage")
    return data


def load_profile():
    data = load_versioned(ROOT / "configs/data-vault/builder-2020.json", "bisect-builder-profile-v1")
    if data.get("year") != 2020 or not isinstance(data.get("sources"), list) or len(data["sources"]) != 6:
        raise ValueError("invalid builder profile")
    seen = set()
    for row in data["sources"]:
        if not {"source", "destination", "revision", "sha256", "line_endings"} <= row.keys():
            raise ValueError("missing builder fields")
        for key in ("source", "destination"):
            inside(ROOT, row[key])
        if (not re.fullmatch(r"[0-9a-f]{40}", row["revision"]) or not valid_hash(row["sha256"])
                or row["line_endings"] not in ("lf", "crlf") or row["destination"].casefold() in seen):
            raise ValueError("invalid builder source")
        seen.add(row["destination"].casefold())
    return data


def sha256(path):
    with path.open("rb") as handle:
        result = hashlib.sha256()
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            result.update(block)
        return result.hexdigest()


def inside(root, relative):
    rel = PurePosixPath(relative)
    if not relative or relative == "." or rel.as_posix() != relative or rel.is_absolute() or ".." in rel.parts or "\\" in relative or ":" in relative:
        raise ValueError(f"unsafe relative path: {relative}")
    path = root.joinpath(*rel.parts)
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError(f"path escapes root through a link: {relative}")
    return path


def vault_root(explicit=None):
    value = explicit or os.environ.get("BISECT_DATA_VAULT")
    if not value and LOCAL.is_file():
        data = load_versioned(LOCAL, "bisect-local-vault-v1")
        value = data.get("vault")
        if not isinstance(value, str) or not value:
            raise ValueError("invalid local vault path")
    if not value:
        raise ValueError("Set --vault, BISECT_DATA_VAULT, or run configure --vault PATH")
    path = Path(value).expanduser().resolve()
    if path == Path(path.anchor) or path == ROOT or path == Path.home():
        raise ValueError("Choose a dedicated project vault directory, not a drive/home/repository root")
    return path


def write_new(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".bisect-json-", dir=path.parent) as temp:
        staged = Path(temp) / "receipt"
        with staged.open("x", encoding="utf-8", newline="\n") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        publish_new(staged, path)


def publish_new(staged, destination):
    # Windows rename refuses an existing target, including on exFAT. POSIX
    # hard-link creation is atomic/no-clobber; unsupported filesystems fail safe.
    if os.name == "nt":
        os.rename(staged, destination)
    else:
        os.link(staged, destination)


def selected(args):
    data = load_catalog()
    unknown = set(args.state or []) - {r["state"] for r in data["items"]}
    if unknown:
        raise ValueError("unknown States: " + ", ".join(sorted(unknown)))
    rows = [r for r in data["items"] if (not args.state or r["state"] in args.state)
            and (not args.year or r["year"] == args.year)
            and (args.group == "all" or r["group"] == args.group or (args.group == "sources" and r["group"] != "contexts"))]
    if not rows:
        raise ValueError("no catalog entries match selection")
    return rows


def inspect(vault, rows, rehash=False):
    result = []
    for row in rows:
        path = inside(vault, row["path"])
        status = "missing"
        if path.is_file():
            status = "size-mismatch" if path.stat().st_size != row["bytes"] else "present-size-only"
            if rehash and status == "present-size-only":
                status = "verified" if sha256(path) == row["sha256"] else "hash-mismatch"
        result.append({"id": row["id"], "path": row["path"], "status": status})
    return result


def copy_checked(source, destination, expected_hash, expected_bytes):
    if source.stat().st_size != expected_bytes or sha256(source) != expected_hash:
        raise ValueError("source hash/size mismatch; nothing admitted")
    if destination.exists():
        if destination.is_file() and destination.stat().st_size == expected_bytes and sha256(destination) == expected_hash:
            return "already-verified"
        raise ValueError(f"existing destination differs; not overwritten: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".bisect-admit-", dir=destination.parent) as temp:
        staged = Path(temp) / "candidate"
        with staged.open("xb") as output, source.open("rb") as input_file:
            shutil.copyfileobj(input_file, output)
            output.flush()
            os.fsync(output.fileno())
        if staged.stat().st_size != expected_bytes or sha256(staged) != expected_hash:
            raise ValueError("copy verification failed; nothing admitted")
        publish_new(staged, destination)
    return "copied-verified"


def census_url(url):
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https" or parsed.hostname != "www2.census.gov" or parsed.username or parsed.password or parsed.port not in (None, 443):
        raise ValueError("catalog download must use official Census HTTPS")


class CensusRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        census_url(newurl)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def open_download(request, timeout):
    return urllib.request.build_opener(CensusRedirect()).open(request, timeout=timeout)


def bounded_copy(source, target, max_bytes, deadline):
    total = 0
    while True:
        if time.monotonic() >= deadline:
            raise ValueError("download deadline exceeded")
        block = source.read(min(1024 * 1024, max_bytes - total + 1))
        if not block:
            break
        total += len(block)
        if total > max_bytes:
            raise ValueError("download byte limit exceeded")
        target.write(block)


def require_space(root, needed):
    probe = root
    while not probe.exists():
        probe = probe.parent
    if shutil.disk_usage(probe).free < needed:
        raise ValueError(f"insufficient free space: need at least {needed} bytes")


def fetch(vault, rows, apply=False, max_bytes=8 * 1024**3, timeout=900):
    if max_bytes <= 0 or timeout <= 0:
        raise ValueError("download limits must be positive")
    missing = [r for r in rows if not inside(vault, r["path"]).exists()]
    unsupported = [r["id"] for r in missing if "url" not in r]
    if unsupported:
        raise ValueError("Derived contexts are rebuilt, not downloadable: " + ", ".join(unsupported))
    if not apply:
        return {"dry_run": True, "downloads": [{"id": r["id"], "url": r["url"]} for r in missing],
                "max_bytes_per_download": max_bytes, "deadline_seconds_per_download": timeout}
    # Rehash existing selected data rather than silently trusting it.
    for result in inspect(vault, rows, True):
        if result["status"] not in ("missing", "verified"):
            raise ValueError(str(result))
    vault.mkdir(parents=True, exist_ok=True)
    results = []
    for url in sorted({r["url"] for r in missing}):
        census_url(url)
        group = [r for r in missing if r["url"] == url]
        ceiling = min(max_bytes, max(r["bytes"] for r in group)) if all("member" not in r for r in group) else max_bytes
        require_space(vault, ceiling + 3 * sum(r["bytes"] for r in group))
        with tempfile.TemporaryDirectory(prefix="bisect-download-", dir=vault) as temp:
            archive = Path(temp) / "download.zip"
            request = urllib.request.Request(url, headers={"User-Agent": "BISECT-data-vault/1"})
            deadline = time.monotonic() + timeout
            with open_download(request, timeout=min(30, timeout)) as response, archive.open("xb") as out:
                census_url(response.url)
                bounded_copy(response, out, ceiling, deadline)
            # Validate every requested member before admitting any from this download.
            staged = []
            for index, row in enumerate(group):
                source = archive
                if "member" in row:
                    source = Path(temp) / str(index)
                    with zipfile.ZipFile(archive) as z:
                        matches = [i for i in z.infolist() if PurePosixPath(i.filename).name == row["member"] and not i.is_dir()]
                        if len(matches) != 1 or matches[0].file_size != row["bytes"]:
                            raise ValueError("missing, duplicate or wrong-sized archive member")
                        with z.open(matches[0]) as inp, source.open("xb") as out:
                            shutil.copyfileobj(inp, out)
                if source.stat().st_size != row["bytes"] or sha256(source) != row["sha256"]:
                    raise ValueError(f"download hash/size mismatch: {row['id']}")
                staged.append((row, source))
            for row, source in staged:
                status = copy_checked(source, inside(vault, row["path"]), row["sha256"], row["bytes"])
                results.append({"id": row["id"], "status": status})
    return {"dry_run": False, "results": results}


def link_directory(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    if os.name == "nt":
        quote = lambda p: "'" + str(p).replace("'", "''") + "'"
        subprocess.run(["powershell", "-NoProfile", "-NonInteractive", "-Command",
                        "New-Item -ItemType Junction -Path " + quote(target) + " -Value " + quote(source) + " | Out-Null"], check=True)
    else:
        target.symlink_to(source, target_is_directory=True)


def is_directory_link(path):
    if path.is_symlink():
        return True
    # Path.is_junction is unavailable on Python 3.11.
    return os.name == "nt" and bool(getattr(path.lstat(), "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT)


def link_plan(vault, repo=ROOT, apply=False, repair=False):
    if repair and not apply:
        raise ValueError("repair requires --apply")
    ownership_path = repo / ".bisect-vault-links.json"
    ownership = load_versioned(ownership_path, "bisect-owned-links-v1") if ownership_path.exists() else {"schema": "bisect-owned-links-v1", "links": {}}
    if not isinstance(ownership.get("links"), dict):
        raise ValueError("invalid link ownership record")
    mappings = [(f"data/{y}/certified", f"derived-data/census/{y}/certified") for y in (2000, 2010, 2020)]
    mappings += [("data/2020/tiger/blocks", "source-data/census/2020/tiger/blocks"),
                 ("data/2020/redistricting", "source-data/census/2020/redistricting"),
                 ("data/enacted_districts", "source-data/external/enacted_districts"),
                 ("runs/nrs-v0.3/neutral-analysis/national-2020", "runs/nrs-v0.3-replay/external-replay-2026-09-09-r6/national-2020")]
    result = []
    for local, remote in mappings:
        source = inside(vault, remote)
        target = repo / local
        if not target.parent.resolve().is_relative_to(repo.resolve()):
            raise ValueError(f"local parent escapes repository: {target}")
        # Repair only links created/recorded by this tool, never user directories.
        if os.path.lexists(target):
            status = "already-linked" if target.resolve() == source.resolve() and source.is_dir() else "existing-local-path-retained"
            old = ownership["links"].get(local)
            if repair and status != "already-linked" and old and is_directory_link(target) and str(target.resolve()) == old:
                if not source.is_dir():
                    status = "missing-vault-directory"
                else:
                    if target.is_symlink():
                        target.unlink()
                    else:
                        os.rmdir(target)  # Removes the verified junction only, not its contents.
                    try:
                        link_directory(source, target)
                    except Exception:
                        if Path(old).is_dir() and not os.path.lexists(target):
                            link_directory(Path(old), target)
                        raise
                    status = "repaired"
                    ownership["links"][local] = str(source.resolve())
        elif not source.is_dir():
            status = "missing-vault-directory"
        else:
            # Reject a parent junction pointing outside the clone.
            if not target.parent.resolve().is_relative_to(repo.resolve()):
                raise ValueError(f"local parent escapes repository: {target}")
            status = "would-link"
            if apply:
                link_directory(source, target)
                status = "linked"
                ownership["links"][local] = str(source.resolve())
        result.append({"repo_path": local, "vault_path": remote, "status": status})
        if apply and status in ("linked", "repaired"):
            # Only this small, schema-validated machine-owned record is replaced.
            with tempfile.TemporaryDirectory(prefix=".bisect-links-", dir=repo) as temp:
                staged = Path(temp) / "links.json"
                write_new(staged, ownership)
                os.replace(staged, ownership_path)
    return result


def ri_replay(vault, name):
    """Replay unmodified engines with resolved inputs, not rewritten old manifests."""
    sys.path.insert(0, str(ROOT / "scripts/research"))
    from run_ri_adjacent_swaps import improve_swaps
    baseline_path = ROOT / "docs/experiments/ri-boundary-budget-2020/analysis.json"
    expected_path = ROOT / "docs/experiments/ri-adjacent-swaps-2020/analysis.json"
    baseline = json.loads(baseline_path.read_text())
    expected = json.loads(expected_path.read_text())
    if ([r["alpha"] for r in baseline["results"]] != ["0", "0.5", "1", "2", "4", "8"]
            or [r["alpha"] for r in baseline["results"]] != [r["alpha"] for r in expected["results"]]):
        raise ValueError("baseline/expected alpha coverage mismatch")
    catalog = load_catalog()
    row = next(r for r in catalog["items"] if r["id"] == "context-2020-RI")
    context_path = inside(vault, row["path"])
    if sha256(context_path) != row["sha256"]:
        raise ValueError("RI context hash mismatch")
    output = inside(vault, "runs/portable-replay/" + name + "/analysis.json")
    if output.parent.exists():
        raise ValueError("choose a new run name")
    context = json.loads(context_path.read_text())
    results = []
    for seed, old in zip(baseline["results"], expected["results"], strict=True):
        if hashlib.sha256(bytes(seed["assignment"])).hexdigest() != seed["assignment_sha256"]:
            raise ValueError("baseline assignment hash mismatch")
        result = improve_swaps(context, seed["assignment"], seed["alpha"])
        if result["assignment_sha256"] != old["assignment_sha256"] or result["final"] != old["final"] or result["status"] != old["status"]:
            raise ValueError("portable replay differs from recorded result")
        results.append({"alpha": result["alpha"], "assignment_sha256": result["assignment_sha256"], "matches": True})
    write_new(output, {"schema": "portable-ri-replay-v1", "context_sha256": row["sha256"],
                      "baseline_sha256": sha256(baseline_path), "expected_sha256": sha256(expected_path),
                      "git_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                      "driver_sha256": sha256(Path(__file__)), "results": results,
                      "engine_sources": {str(p.relative_to(ROOT)).replace("\\", "/"): sha256(p) for p in
                                         [ROOT / ("scripts/research/" + file + ".py") for file in
                                          ("run_ri_adjacent_swaps", "run_ri_boundary_moves", "run_ri_county_tree", "run_ri_population_band", "run_multiroot_county_diagnostic", "analyze_dfs_county_response")]},
                      "scope": "Semantic replay with relocated inputs; not byte-identical historical manifest replay."})
    return {"status": "passed", "output": str(output), "arms": len(results)}


def runtime_inventory():
    native = {}
    for name in ("shapely", "pyproj"):
        try:
            module = __import__(name)
            native[name] = {key: str(getattr(module, key)) for key in ("geos_version_string", "proj_version_str") if hasattr(module, key)}
        except ImportError:
            native[name] = "not-installed"
    return {"python": sys.version, "executable": sys.executable, "platform": platform.platform(),
            "packages": sorted([{"name": d.metadata["Name"], "version": d.version} for d in importlib.metadata.distributions()], key=lambda d: d["name"].lower()),
            "native": native}


def build_2020(vault, state, name, apply=False, timeout=3600):
    """Restore the exact historical builder from Git; admit only matching output."""
    if timeout <= 0:
        raise ValueError("build timeout must be positive")
    catalog = load_catalog()["items"]
    profile = load_profile()
    expected = next((r for r in catalog if r["id"] == "context-2020-" + state), None)
    if expected is None:
        raise ValueError("unknown State")
    destination = inside(vault, expected["path"])
    if destination.exists():
        if sha256(destination) != expected["sha256"]:
            raise ValueError("existing context differs; not overwritten")
        return {"status": "already-verified", "path": str(destination)}
    sources = [r for r in catalog if r["year"] == 2020 and r["state"] == state and r["group"] in ("blocks", "population")]
    if any(r["status"] != "verified" for r in inspect(vault, sources, True)):
        raise ValueError("fetch and verify 2020 blocks and population sources first")
    workspace = inside(vault, "runs/context-builds/" + name)
    if workspace.exists():
        raise ValueError("choose a new build run name")
    if not apply:
        return {"dry_run": True, "workspace": str(workspace), "output": str(destination)}
    require_space(vault, 8 * sum(r["bytes"] for r in sources) + 4 * expected["bytes"])
    restored = []
    for row in profile["sources"]:
        data = subprocess.check_output(["git", "show", row["revision"] + ":" + row["source"]], cwd=ROOT)
        data = data.replace(b"\r\n", b"\n")
        if row["line_endings"] == "crlf":
            data = data.replace(b"\n", b"\r\n")
        if hashlib.sha256(data).hexdigest() != row["sha256"]:
            raise ValueError("historical builder source mismatch")
        restored.append((row, data))
    # Extraction is flat and validates each existing file against its ZIP member.
    archive_row = next(r for r in sources if r["group"] == "blocks")
    archive = inside(vault, archive_row["path"])
    shape_root = inside(vault, "source-data/census/2020/tiger/blocks/" + archive.stem)
    with zipfile.ZipFile(archive) as z:
        for info in z.infolist():
            if info.is_dir():
                continue
            if PurePosixPath(info.filename).name != info.filename or "\\" in info.filename or ":" in info.filename:
                raise ValueError("unexpected nested/unsafe shapefile archive member")
            with tempfile.TemporaryDirectory(prefix="bisect-extract-", dir=vault) as temp:
                file = Path(temp) / "member"
                with z.open(info) as inp, file.open("xb") as out:
                    shutil.copyfileobj(inp, out)
                copy_checked(file, inside(shape_root, info.filename), sha256(file), info.file_size)
    workspace.mkdir(parents=True)
    for row, data in restored:
        target = inside(workspace, row["destination"])
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as out:
            out.write(data)
    link_directory(inside(vault, "source-data/census/2020/tiger/blocks"), workspace / "data/2020/tiger/blocks")
    link_directory(inside(vault, "source-data/census/2020/redistricting"), workspace / "data/2020/redistricting")
    sys.path.insert(0, str(ROOT / "scripts"))
    from config.download_sources import STATE_FIPS, STATE_NAMES
    candidate = workspace / "candidate.rctx"
    environment = dict(os.environ, OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    command = [sys.executable, str(workspace / "scripts/research/build_state_block_rctx.py"),
               "--state-code", state, "--state-fips", STATE_FIPS[state], "--state-name", STATE_NAMES[state].replace("_", " ").title(),
               "--rctx", "candidate.rctx", "--report", "report.json", "--manifest", "manifest.json"]
    with (workspace / "build.log").open("x", encoding="utf-8") as log:
        try:
            result = subprocess.run(command, cwd=workspace, env=environment, stdout=log, stderr=subprocess.STDOUT, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            raise ValueError(f"build deadline exceeded; retained workspace: {workspace}") from error
    if result.returncode:
        raise ValueError(f"builder failed; retained log: {workspace / 'build.log'}")
    if candidate.stat().st_size != expected["bytes"] or sha256(candidate) != expected["sha256"]:
        raise ValueError(f"rebuilt context differs (check geospatial dependency versions); candidate retained at {candidate}")
    receipt = {"schema": "bisect-context-admission-v2", "status": "verified", "context": expected,
               "context_sha256": expected["sha256"], "catalog_sha256": sha256(CATALOG), "inputs": sources,
               "driver_sha256": sha256(Path(__file__)),
               "helper_sha256": sha256(ROOT / "scripts/config/download_sources.py"),
               "builder_profile_sha256": sha256(ROOT / "configs/data-vault/builder-2020.json"),
               "builder_sources": profile["sources"], "command": command,
               "resolved_state": {"code": state, "fips": STATE_FIPS[state], "name": STATE_NAMES[state].replace("_", " ").title()},
               "runtime": runtime_inventory(), "thread_environment": {k: environment[k] for k in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")},
               "timeout_seconds": timeout,
               "artifacts": {name: sha256(workspace / name) for name in ("manifest.json", "report.json", "build.log")}}
    # Materialize evidence before publishing the canonical context; no success
    # receipt is written on a failed builder or mismatched output.
    copy_checked(candidate, destination, expected["sha256"], expected["bytes"])
    write_new(workspace / "admission.json", receipt)
    return {"status": "rebuilt-and-verified", "path": str(destination), "workspace": str(workspace)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    for action in ("configure", "status", "link", "fetch", "build-2020", "ri-replay"):
        sub = actions.add_parser(action)
        sub.add_argument("--vault", help="dedicated project vault directory")
        if action in ("status", "fetch"):
            sub.add_argument("--group", choices=("all", "sources", "contexts", "blocks", "population", "enacted"), default="all")
            sub.add_argument("--year", type=int, choices=(2000, 2010, 2020))
            sub.add_argument("--state", nargs="+", type=str.upper)
        if action == "status":
            sub.add_argument("--hash", action="store_true")
        if action in ("link", "fetch", "build-2020"):
            sub.add_argument("--apply", action="store_true")
        if action == "link":
            sub.add_argument("--repair", action="store_true", help="repair only tool-owned stale links; requires --apply")
        if action == "fetch":
            sub.add_argument("--max-bytes", type=int, default=8 * 1024**3, help="per-download byte ceiling; raise explicitly for large archives")
            sub.add_argument("--timeout", type=float, default=900, help="per-download deadline in seconds")
        if action == "build-2020":
            sub.add_argument("--state", required=True, type=str.upper)
            sub.add_argument("--timeout", type=float, default=3600, help="builder wall-time limit in seconds")
        if action in ("build-2020", "ri-replay"):
            sub.add_argument("--run-name", required=True)
    args = parser.parse_args()
    vault = vault_root(args.vault)
    if args.action == "configure":
        if not vault.is_dir():
            raise ValueError("vault must exist before recording this machine's configuration")
        write_new(LOCAL, {"schema": "bisect-local-vault-v1", "vault": str(vault)})
        result = {"configured": str(vault), "local_only": str(LOCAL)}
    elif args.action == "link":
        result = link_plan(vault, apply=args.apply, repair=args.repair)
    elif args.action == "fetch":
        result = fetch(vault, selected(args), args.apply, max_bytes=args.max_bytes, timeout=args.timeout)
    elif args.action == "ri-replay":
        result = ri_replay(vault, args.run_name)
    elif args.action == "build-2020":
        result = build_2020(vault, args.state, args.run_name, args.apply, timeout=args.timeout)
    else:
        rows = inspect(vault, selected(args), args.hash)
        result = {"vault": str(vault), "files": rows, "counts": {s: sum(r["status"] == s for r in rows) for s in sorted({r["status"] for r in rows})}}
    print(json.dumps(result, indent=2))
    if args.action == "status" and any(r["status"] not in ("verified", "present-size-only") for r in rows):
        return 1
    if args.action == "link" and any(r["status"] not in ("already-linked", "linked", "repaired", "would-link") for r in result):
        return 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, subprocess.CalledProcessError, zipfile.BadZipFile) as error:
        print(f"vault error: {error}", file=sys.stderr)
        sys.exit(2)
