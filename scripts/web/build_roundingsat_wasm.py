"""Build the pinned external solver from locally acquired toolchain sources.

No system installation, network acquisition, or upstream source mutation occurs.
The resulting module must be instantiated afresh for each solve (upstream globals).
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

COMMIT = "d4edbf7908a9bb951fd181940919e0f3ac7ab1ee"
BOOST_SHA = "2575e74ffc3ef1cd0babac2c1ee8bdb5782a0ee672b1912da40e5b4b591ca01f"
SDK_VERSION = "6.0.12"
SOURCES = """Constr ConstrExp ConstrExpPool ConstrSimple Env Logger LpSolver
ProofBuffer Solver SolverStructs Stats io objrewrite optimize parsing quit
roundingsat run""".split()
LICENSE_SOURCES = "gpl_3_0 apache_2 roundingsat MIT licenses boost lgpl_3_0".split()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain", type=Path, default=Path("target/proof-toolchain"))
    parser.add_argument("--output", type=Path, default=Path("target/roundingsat-wasm"))
    args = parser.parse_args()
    base = args.toolchain.resolve()
    source = base / "roundingsat"
    commit = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    if commit != COMMIT or subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip():
        raise ValueError("RoundingSat requires the clean pinned upstream checkout")
    if sha(base / "boost_1_86_0.tar.gz") != BOOST_SHA:
        raise ValueError("Boost archive checksum mismatch")
    emcc = base / "emsdk/upstream/emscripten/em++.py"
    env = dict(os.environ, EM_CONFIG=str(base / "emsdk/.emscripten"))
    version = subprocess.check_output([sys.executable, str(emcc), "--version"], env=env, text=True)
    if f" {SDK_VERSION} " not in version.splitlines()[0]:
        raise ValueError("Unexpected Emscripten version")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    include = out / "include"
    include.mkdir(exist_ok=True)
    (include / "CMakeConfig.hpp").write_text("#pragma once\n// No compression dependencies.\n", encoding="utf-8")
    (include / "version.hpp").write_text(f'#pragma once\n#define GIT_COMMIT_HASH "{COMMIT}"\n#define GIT_BRANCH "pinned-wasm"\n', encoding="utf-8")
    files = [source / "src" / f"{name}.cpp" for name in SOURCES]
    files += [source / "src/used_licenses" / f"{name}.cpp" for name in LICENSE_SOURCES]
    clock_adapter = Path(__file__).with_name('roundingsat_wasm_clock.cpp').resolve()
    files.append(clock_adapter)
    # Upstream already supports platforms without intrinsic int128. Select that
    # exact-arithmetic Boost path: libc++ does not overload std::abs(__int128).
    flags = ["-std=c++20", "-O2", "-DNDEBUG", "-U__SIZEOF_INT128__", "-Wno-deprecated-literal-operator", "-fexceptions", "-I" + str(include), "-I" + str(source / "src"), "-I" + str(base / "boost"),
             "-sMODULARIZE=1", "-sEXPORT_ES6=1", "-sENVIRONMENT=web,worker,node",
             "-sALLOW_MEMORY_GROWTH=1", "-sMAXIMUM_MEMORY=536870912", "-sSTACK_SIZE=8388608",
             "-sINVOKE_RUN=0", "-sEXIT_RUNTIME=1", "-sDYNAMIC_EXECUTION=0",
             '-sEXPORTED_RUNTIME_METHODS=["FS","callMain"]', "-sFORCE_FILESYSTEM=1",
             '-sEXPORTED_FUNCTIONS=["_main","_bisect_solver_elapsed_seconds"]', '-Wl,--wrap=getrusage']
    subprocess.run([sys.executable, str(emcc), *flags, *map(str, files), "-o", str(out / "roundingsat.mjs")], env=env, check=True)
    receipt = {"schema": "bisect-roundingsat-wasm-build-v1", "upstream": "https://gitlab.com/MIAOresearch/software/roundingsat",
               "commit": COMMIT, "emscripten": SDK_VERSION, "boost_version": "1.86.0", "boost_archive_sha256": BOOST_SHA,
               "source_patches": [], "lp": False, "gmp": False, "compression": False,
               "clock": "monotonic-elapsed-time", "clock_adapter_sha256": sha(clock_adapter),
               "memory_limit_bytes": 536870912, "fresh_module_per_solve": True,
               "flags": flags, "files": {name: {"sha256": sha(out / name), "bytes": (out / name).stat().st_size}
                                        for name in ["roundingsat.mjs", "roundingsat.wasm"]}}
    (out / "build-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(receipt["files"], indent=2))


if __name__ == "__main__":
    main()
