"""Create a map or rebuild ZIP, extract safely, then verify relocated rendering."""
import argparse
import json
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import zipfile
from workspace import REPO, sha, write_json, relative


def extract(archive, destination):
    with zipfile.ZipFile(archive) as z:
        names = set()
        for member in z.infolist():
            path = relative(member.filename)
            if member.filename.lower() in names or stat.S_ISLNK(member.external_attr >> 16):
                raise ValueError('Duplicate or linked ZIP member')
            names.add(member.filename.lower())
            if member.is_dir(): raise ValueError('Release ZIP contains unexpected directory member')
        if z.testzip() is not None: raise ValueError('ZIP CRC failure')
        destination.mkdir(parents=True, exist_ok=False)
        for member in z.infolist():
            target = destination / relative(member.filename); target.parent.mkdir(parents=True, exist_ok=True)
            with z.open(member) as source, target.open('xb') as out: shutil.copyfileobj(source, out)


def package(source, run, archive, kind):
    if archive.exists(): raise FileExistsError(archive)
    if json.loads((source / 'provenance/output-checks.json').read_text())['status'] != 'passed':
        raise ValueError('Data and PDF checks must pass before packaging')
    archive.parent.mkdir(parents=True, exist_ok=True)
    staging = archive.with_suffix('.staging'); staging.mkdir(exist_ok=False)
    for path in source.rglob('*'):
        if path.is_symlink() or path.is_junction(): raise ValueError('Links are not portable package files')
        if path.is_file():
            if kind == 'read' and path.relative_to(source).parts[0] not in ('exports', 'provenance', 'environment.json'): continue
            target = staging / path.relative_to(source); target.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(path, target)
    scripts = staging / 'scripts/publication'; scripts.mkdir(parents=True)
    include = ['run_qgis.py', 'export_map.py', 'workspace.py']
    if kind == 'read': include = []
    if kind == 'rebuild':
        include += ['build_map.py', 'validate_map.py']
        shutil.copytree(run / 'inputs', staging / 'inputs'); shutil.copy2(run / 'selection.json', staging / 'selection.json')
    for name in include: shutil.copy2(REPO / 'scripts/publication' / name, scripts / name)
    licenses = staging / 'LICENSES'; licenses.mkdir(); shutil.copy2(REPO / 'LICENSE', licenses / 'BISECT.txt')
    (licenses / 'CENSUS.md').write_text('U.S. Census Bureau, 2020 TIGER/Line and PL 94-171.\nFederal Census source data retain their public-domain status.\nSource geometry terms are also included in the TIGER ZIP XML metadata in the rebuild package.\nhttps://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.2020.html\nBISECT authored content uses the separate terms in BISECT.txt.\n', encoding='utf-8')
    instructions = '''# BISECT Rhode Island 2020 publication pilot

Unzip the entire package, then open project.qgs in QGIS Desktop. Keep its
attachment companion and data folder alongside it. Project > Layouts offers
overview, district-1 and district-2. PDFs and PNGs are already in exports.

Tested on Windows with QGIS 3.44.12. No ArcGIS, network, external drive, Rust,
plugins or sibling repositories are needed to open and edit the map. Use
QGIS's matching Python/GDAL for the scripts; do not pip-install a replacement
GIS runtime. Python 3.12 runs the included Windows launcher. Other platforms
have not been tested; the Windows launcher does not support them.

Export saved layouts into a new folder from this package directory:

    py -3.12 scripts/publication/run_qgis.py scripts/publication/export_map.py --package . --output exports-new

Cartography can be edited in QGIS. Census units, district assignments and
population fields preserve an existing research candidate. Changing an
assignment makes a new plan; manually editing units does not automatically
update dissolved district outlines, population summaries or layout text.
Rebuild those derivatives and validate the new candidate before sharing.

This is the reference baseline candidate, not an enacted map or a new legal,
fairness or optimality finding. Source package verification was recorded as
pass; the publication checks cover data preservation and rendering, not a
fresh solver certification. Water-only blocks are overlaid in blue-gray;
the data still retain their full geometry and assignments. Population is
2020 PL 94-171 total resident population. Full identifiers/hashes and status
are in provenance. Source geometry is NAD83; display is UTM zone 19N.

Licenses differ for software, original BISECT content and Census data; read
LICENSES before redistribution. This ZIP is a local release artifact, not
evidence of an external upload. SHA256SUMS covers every other packaged file.
'''
    if kind == 'rebuild':
        instructions += '''
## Rebuild the publication (not the district solver)

Exact inputs and builder source are included. From this folder, use new output
directories (existing projects are never overwritten):

    py -3.12 scripts/publication/run_qgis.py scripts/publication/build_map.py --run . --output rebuilt
    py -3.12 scripts/publication/run_qgis.py scripts/publication/export_map.py --package rebuilt --output rebuilt/exports
    py -3.12 scripts/publication/run_qgis.py scripts/publication/validate_map.py --package rebuilt --run .

The final validation command additionally needs Poppler's pdfinfo/pdftotext
on PATH. environment.json records exact tested versions. Historical source
evidence is preserved byte-for-byte. selection.json's vault-relative source
paths describe provenance; rebuilding reads only its local inputs paths.
'''
    if kind == 'read':
        instructions = '''# BISECT Rhode Island 2020 atlas — reader edition

Open exports/overview.pdf, exports/district-1.pdf and exports/district-2.pdf
with any PDF viewer. Matching PNG previews are included. No GIS installation
is required. This is a reference baseline candidate, not an enacted map or
a new legal, fairness or optimality finding. The mapped population is 2020
PL 94-171 total resident population; boundaries use 2020 TIGER/Line.

Source identities, hashes and publication checks are in provenance; source
data and editable projects are in the separate map/rebuild editions. Read
LICENSES for attribution and the separate software/content/data terms.
SHA256SUMS covers every other packaged file.
'''
    (staging / 'README.md').write_text(instructions, encoding='utf-8')
    write_json(staging / 'provenance/builder-snapshot.json', {'schema': 'bisect-publication-builder-v1',
        'repository_commit': json.loads((run / 'state.json').read_text())['repository_commit'],
        'source_status': 'uncommitted pilot source snapshot included', 'scripts': {n: sha(scripts / n) for n in include}})
    files = sorted(p for p in staging.rglob('*') if p.is_file())
    (staging / 'SHA256SUMS').write_text(''.join(f'{sha(p)}  {p.relative_to(staging).as_posix()}\n' for p in files), encoding='utf-8')
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for path in sorted(p for p in staging.rglob('*') if p.is_file()): z.write(path, path.relative_to(staging).as_posix())
    relocated = archive.with_suffix('.extracted'); extract(archive, relocated)
    sums = (relocated / 'SHA256SUMS').read_text().splitlines()
    assert len(sums) + 1 == len([p for p in relocated.rglob('*') if p.is_file()])
    for line in sums:
        digest, name = line.split('  ', 1); assert sha(relocated / relative(name)) == digest
    expected = json.loads((relocated / 'provenance/verification.json').read_text())['expected_pages']
    if kind != 'read':
        renders = archive.with_suffix('.rendered')
        subprocess.run([sys.executable, str(relocated / 'scripts/publication/run_qgis.py'), str(relocated / 'scripts/publication/export_map.py'),
                        '--package', str(relocated), '--output', str(renders)], check=True)
        assert sorted(x.stem for x in renders.glob('*.png')) == sorted(expected)
        for name in expected: assert sha(renders / (name + '.png')) == sha(source / 'exports' / (name + '.png'))
    else:
        for name in expected:
            for ext in ('.pdf', '.png'): assert sha(relocated / 'exports' / (name+ext)) == sha(source / 'exports' / (name+ext))
    if kind == 'rebuild':
        rebuilt = archive.with_suffix('.rebuilt')
        launcher = [sys.executable, str(relocated / 'scripts/publication/run_qgis.py')]
        subprocess.run(launcher + [str(relocated / 'scripts/publication/build_map.py'), '--run', str(relocated), '--output', str(rebuilt)], check=True)
        subprocess.run(launcher + [str(relocated / 'scripts/publication/export_map.py'), '--package', str(rebuilt), '--output', str(rebuilt / 'exports')], check=True)
        subprocess.run(launcher + [str(relocated / 'scripts/publication/validate_map.py'), '--package', str(rebuilt), '--run', str(relocated)], check=True)
        for name in expected: assert sha(rebuilt / 'exports' / (name + '.png')) == sha(source / 'exports' / (name + '.png'))
    write_json(archive.with_suffix('.verification.json'), {'schema': 'bisect-publication-package-v1', 'status': 'verified',
        'kind': kind, 'zip_sha256': sha(archive), 'bytes': archive.stat().st_size, 'files': len(sums)+1,
        'relocated_pages_byte_identical': expected, 'relocated_reexport_tested': kind != 'read', 'rebuild_from_inputs_tested': kind == 'rebuild'})
    print(f'Verified {kind}: {archive} ({archive.stat().st_size:,} bytes)', flush=True)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--source', type=Path, required=True); p.add_argument('--run', type=Path, required=True)
    p.add_argument('--archive', type=Path, required=True); p.add_argument('--kind', choices=['read', 'map', 'rebuild'], required=True)
    args = p.parse_args(); package(args.source.resolve(), args.run.resolve(), args.archive.resolve(), args.kind)
