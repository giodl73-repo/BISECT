"""Prepare a Wrangler project from an existing static build without deploying."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil


ROOT = Path(__file__).resolve().parents[2]
MAX_ASSET = 25 * 1024**2
MAX_ASSETS = 20_000
ASSET_PREFIX = "_bisect_wasm_transport"
WORKER_FILES = ("source-worker.mjs", "adjacency-release-bridge.mjs", "wasm-static-transport.mjs")
HEADER = b"\x00asm\x01\x00\x00\x00"


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def pin(path):
    with path.open("rb") as stream:
        return {"bytes": path.stat().st_size, "sha256": hashlib.file_digest(stream, "sha256").hexdigest()}


def json_write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def inspect(source):
    entries = {}
    for path in sorted(source.rglob("*")):
        name = path.relative_to(source).as_posix()
        require(not path.is_symlink() and not path.is_junction() and path.resolve().is_relative_to(source),
                f"Linked asset: {name}")
        require(name.split("/")[0] != ASSET_PREFIX, f"Reserved asset namespace: {name}")
        require(path.is_dir() or path.is_file(), f"Unsupported asset: {name}")
        if not path.is_file():
            continue
        size = path.stat().st_size
        if size > MAX_ASSET:
            require(path.name == "bisect_wasm.wasm" and
                    re.fullmatch(r"(?:[A-Za-z0-9_-]+/)*bisect_wasm\.wasm", name),
                    f"Static asset exceeds 25 MiB: {name}")
        entries[name] = pin(path)
    require("index.html" in entries and any(n.split("/")[-1] == "bisect_wasm.wasm" for n in entries),
            "Select an existing static BISECT build")
    if "offline-manifest.json" in entries:
        offline = json.loads((source / "offline-manifest.json").read_bytes())
        for name, expected in offline["assets"].items():
            require(entries.get(name) == expected, f"Offline manifest differs from input bytes: {name}")
    return entries


def static_redirects(entries):
    # Serve explicit index.html to the redirect-rejecting offline installer.
    # Keep clean navigation URLs on Static Assets, outside the Worker quota.
    rules = {}
    for name in entries:
        if not re.fullmatch(r"(?:[A-Za-z0-9_-]+/)*[A-Za-z0-9_-]+\.html", name):
            continue
        if name == "index.html" or name.endswith("/index.html"):
            directory = "/" + name.removesuffix("index.html")
            rules[directory] = f"/{name} 200"
            if directory != "/" and directory.strip("/") not in entries:
                rules[directory.rstrip("/")] = f"{directory} 307"
        elif name.removesuffix(".html") not in entries:
            rules.setdefault("/" + name.removesuffix(".html"), f"/{name} 200")
    require(len(rules) <= 2000, "Too many static HTML aliases")
    return "".join(f"{url} {dest}\n" for url, dest in sorted(rules.items()))


def package(source, output, worker_name):
    require(not source.is_symlink() and not source.is_junction(), "Use a real asset directory")
    source = source.resolve(strict=True)
    output = output.resolve()
    require(source.is_dir() and not output.exists() and not output.is_relative_to(source),
            "Use an existing build and a fresh output outside it")
    require(re.fullmatch(r"[a-z0-9][a-z0-9-]{0,62}", worker_name), "Invalid Worker name")
    entries = inspect(source)
    require("_redirects" not in entries, "Input has custom Cloudflare redirects; merge the static HTML aliases explicitly")
    redirects = static_redirects(entries)
    output.mkdir(parents=True)
    assets = output / "assets"
    encoded, routes, stored = {}, {}, {}
    for name, expected in entries.items():
        original = source / name
        if expected["bytes"] > MAX_ASSET:
            raw = original.read_bytes()
            require(raw.startswith(HEADER) and sha(raw) == expected["sha256"], f"Invalid WASM: {name}")
            public = f"{ASSET_PREFIX}/{expected['sha256']}.wasm.gz"
            if public not in stored:
                dest = assets / public
                dest.parent.mkdir(parents=True, exist_ok=True)
                compressed = gzip.compress(raw, compresslevel=9, mtime=0)
                require(len(compressed) <= MAX_ASSET and gzip.decompress(compressed) == raw,
                        f"WASM cannot be packaged within the asset limit: {name}")
                with dest.open("xb") as stream:
                    stream.write(compressed)
                stored[public] = pin(dest)
            routes["/" + name] = "/" + public
            encoded[name] = {"decoded": expected, "asset": public, "gzip": stored[public]}
        else:
            dest = assets / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, dest)
            require(pin(dest) == expected, f"Input changed while staging: {name}")
            stored[name] = expected
    with (assets / "_redirects").open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(redirects)
    stored["_redirects"] = pin(assets / "_redirects")
    require(len(stored) <= MAX_ASSETS, "Static asset count exceeds the free plan limit")
    require(all(v["bytes"] <= MAX_ASSET for v in stored.values()), "Oversized output asset")
    require(all(pin(source / n) == v for n, v in entries.items()), "Input build changed during packaging")
    for name in WORKER_FILES:
        dest = output / "cloudflare" / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / "cloudflare" / name, dest)
    # Preserve the existing Worker API routes and their browser-module imports.
    for src in sorted((ROOT / "web/lab").glob("*.js")):
        dest = output / "web/lab" / src.name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, dest)
    routes_text = json.dumps(routes, sort_keys=True)
    with (output / "cloudflare/index.mjs").open("x", encoding="utf-8", newline="\n") as stream:
        stream.write("import fallback from './source-worker.mjs';\n"
                     "import {hostedWasmTransport} from './wasm-static-transport.mjs';\n"
                     f"export default hostedWasmTransport(fallback, {routes_text});\n")
    template = json.loads((ROOT / "wrangler.cloudflare.jsonc").read_bytes())
    json_write(output / "wrangler.json", {
        "name": worker_name, "compatibility_date": template["compatibility_date"], "main": "./cloudflare/index.mjs",
        "assets": {"directory": "./assets", "binding": "ASSETS", "html_handling": "none",
                   "run_worker_first": [*template["assets"]["run_worker_first"], *sorted(routes)]},
    })
    json_write(output / "package.json", {"private": True, "type": "module"})
    record = {"schema_version": "bisect-cloudflare-static-package-v1", "worker_name": worker_name,
              "assets": stored, "gzip_routes": encoded, "decoded_offline_manifest_preserved": "offline-manifest.json" in entries,
              "deployed": False}
    json_write(output / "hosting-manifest.json", record)
    print(f"Staged {len(stored)} assets and {len(routes)} precompressed WASM routes in {output}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--name", required=True)
    args = parser.parse_args()
    package(args.input, args.output, args.name)
