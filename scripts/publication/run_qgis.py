"""Run a repository Python script with an independent Windows QGIS installation."""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import runpy
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--qgis-root", type=Path, default=None)
    parser.add_argument("--child", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("script", type=Path)
    parser.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.qgis_root:
        root = args.qgis_root.resolve(strict=True)
    else:
        candidates = sorted(Path(os.environ.get("ProgramFiles", r"C:\Program Files")).glob("QGIS *"))
        if len(candidates) != 1:
            parser.error("Specify --qgis-root when there is not exactly one QGIS installation")
        root = candidates[0]
    prefix = root / "apps/qgis-ltr"
    if not prefix.is_dir():
        prefix = root / "apps/qgis"
    python_homes = list((root / "apps").glob("Python3*"))
    if len(python_homes) != 1 or not prefix.is_dir():
        parser.error("Unsupported QGIS installation layout")
    python_home = python_homes[0]
    dll_dirs = [root / "bin", root / "apps/Qt5/bin", prefix / "bin"]
    env = os.environ.copy()
    env.update({
        "PYTHONHOME": str(python_home), "PYTHONPATH": str(prefix / "python"),
        "PYTHONUTF8": "1", "QGIS_PREFIX_PATH": str(prefix),
        "PYTHONUNBUFFERED": "1",
        "QT_QPA_PLATFORM": "offscreen",
        "QT_QPA_FONTDIR": str(Path(os.environ.get("WINDIR", r"C:\Windows")) / "Fonts"),
        "QT_PLUGIN_PATH": str(root / "apps/Qt5/plugins"),
        "GDAL_DATA": str(root / "apps/gdal/share/gdal"),
        "GDAL_DRIVER_PATH": str(root / "apps/gdal/lib/gdalplugins"),
        "PROJ_DATA": str(root / "share/proj"),
        "OGR_SQLITE_JOURNAL": "DELETE",
        "PATH": os.pathsep.join(str(path) for path in dll_dirs) + os.pathsep + os.environ.get("PATH", ""),
    })
    if not args.child:
        command = [str(root / "bin/python.exe"), str(Path(__file__).resolve()),
                   "--qgis-root", str(root), "--child", str(args.script.resolve()), *args.arguments]
        raise SystemExit(subprocess.call(command, env=env))
    os.environ.update(env)
    handles = [os.add_dll_directory(str(path)) for path in dll_dirs if path.is_dir()]
    sys.path.insert(0, str(prefix / "python"))
    sys.path.insert(0, str(args.script.resolve().parent))
    sys.argv = [str(args.script), *args.arguments]
    runpy.run_path(str(args.script), run_name="__main__")
    # Keep DLL directory handles alive until script execution finishes.
    for handle in handles:
        handle.close()


if __name__ == "__main__":
    main()
