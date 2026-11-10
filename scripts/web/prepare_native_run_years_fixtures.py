"""Package small synthetic vintage-specific Census fixtures as VT ZIPs."""
import base64
import gzip
import json
from pathlib import Path
import zipfile

root = Path('target/stream-native-run-years/fixtures')
root.mkdir(parents=True, exist_ok=True)
for year in ['2020', '2010', '2000']:
    files = json.loads(Path(f'target/census-preparation-wasm/{year}-tract/project.bisect').read_text(encoding='utf-8'))['files']
    def raw(source):
        data = base64.b64decode(source['base64'])
        return gzip.decompress(data) if data[:2] == b'\x1f\x8b' else data
    with zipfile.ZipFile(root / f'{year}-tiger.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
        for role in ['shp', 'dbf', 'shx']:
            data = raw(files['tigerInputs'][role])
            if role == 'dbf':
                assert b'44001' in data
                data = data.replace(b'44001', b'50001')
            archive.writestr('units.' + role, data)
        archive.writestr('units.prj', raw(files['tractPreparationSettings']['projection']))
    with zipfile.ZipFile(root / f'{year}-pl94.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
        for role, name in [('geography', 'vtgeo.upl' if year == '2000' else f'vtgeo{year}.pl'), ('population', f'vt00001{year}.pl')]:
            source = files['pl94Inputs'][role]
            if source:
                archive.writestr(name, raw(source).replace(b'44001', b'50001'))
    if year == '2000':
        with zipfile.ZipFile(root / 'ambiguous-2000.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
            data = raw(files['pl94Inputs']['geography']).replace(b'44001', b'50001')
            archive.writestr('vtgeo.upl', data)
            archive.writestr('vtgeo.uf1', data)
print('Prepared synthetic VT tract ZIP pairs for native Run 2020/2010/2000 acquisition')
