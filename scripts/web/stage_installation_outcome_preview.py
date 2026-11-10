"""Stage the passive installation-outcome UI without modifying accepted packages."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(path):
    return dict(path=path.relative_to(ROOT).as_posix(),
                sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                byte_length=path.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suffix', default='preview')
    args = parser.parse_args()
    assert args.suffix in ['preview', 'durable-preview'], 'Choose a declared local candidate'
    assert shutil.disk_usage(ROOT).free > 512 * 1024**2, 'Insufficient staging reserve'
    output = ROOT / 'target/install-outcome' / ('stage-' + str(time.time_ns()))
    output.mkdir(parents=True)
    profiles = {}
    for profile, name in [('default', 'wasm-national-queue-default-preview'),
                          ('c-sidecar', 'wasm-national-queue-preview')]:
        base = ROOT / 'dist' / name
        destination = ROOT / 'dist' / ('wasm-install-outcome-' + profile + '-' + args.suffix)
        assert base.is_dir() and not destination.exists(), destination
        before = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
        assert not any(p.is_symlink() for p in base.rglob('*')), 'No linked directories'
        # Previously tested assets are shared read-only; detach each changed file
        # before writing. Never write through a hardlink into the accepted base.
        shutil.copytree(base, destination, copy_function=os.link)
        source = ROOT / 'web/lab/native-state-output-ui.js'
        for relative in ['native-state-output-ui.js', 'toolkit/native-state-output-ui.js',
                         'installation-outcome-store.js', 'toolkit/installation-outcome-store.js',
                         'offline-manifest.json']:
            target = destination / relative
            temporary = target.with_name(target.name + '.installation-outcome-tmp')
            shutil.copyfile(target if relative == 'offline-manifest.json' else ROOT / 'web/lab' / Path(relative).name, temporary)
            os.replace(temporary, target)
        subprocess.run([sys.executable, 'scripts/web/build_offline_manifest.py', str(destination)],
                       cwd=ROOT, check=True)
        for ref in before:
            assert pin(ROOT / ref['path']) == ref, 'Accepted package was modified'
        for source in (ROOT / 'web/lab').iterdir():
            if source.is_file() and source.name not in ['index.html', 'bisect_wasm.wasm']:
                for directory in [destination, destination / 'toolkit']:
                    assert pin(directory / source.name)['sha256'] == pin(source)['sha256'], source
        profiles[profile] = dict(base=str(base.relative_to(ROOT)),
                                 package=str(destination.relative_to(ROOT)),
                                 engine=pin(destination / 'toolkit/bisect_wasm.wasm'),
                                 ui=pin(destination / 'toolkit/native-state-output-ui.js'),
                                 offline=pin(destination / 'offline-manifest.json'),
                                 base_unchanged=True)
    (output / 'evidence.json').write_text(json.dumps(dict(
        schema_version='bisect-installation-outcome-stage-v1', profiles=profiles,
        source=pin(ROOT / 'web/lab/native-state-output-ui.js'),
        rust_or_engine_changes=False, deployed=False), indent=2) + '\n', encoding='utf-8')
    print(output / 'evidence.json')


if __name__ == '__main__':
    main()
