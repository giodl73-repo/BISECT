"""Check provider ZIP CRCs, exact selected members, and independent block totals."""
import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path
import zipfile


def audit(root, browser):
    results = {}
    evidence = json.loads((browser / 'evidence.json').read_bytes())
    assert evidence['status'] == 'passed-packaged-live-census-vintages'
    for scope, case in evidence['results'].items():
        state, year = scope.split(':')
        archive = root / case['url'].rsplit('/', 1)[-1]
        request = json.loads((browser / f'{state}-{year}-native-request.json').read_bytes())
        assert request['operation'] == 'prepare-pl94-source'
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == case['archive_sha256']
        with zipfile.ZipFile(archive) as z:
            assert z.testzip() is None
            names = z.namelist()
            assert len(set(names)) == len(names)
            members = {}
            for role in ('geography', 'population'):
                selected = request['input'][role]
                if selected is None:
                    assert year == '2000' and role == 'population'
                    continue
                original = selected['name'].removesuffix('.gz')
                decoded = gzip.decompress(base64.b64decode(selected['base64'], validate=True))
                assert decoded == z.read(original)
                members[role] = decoded
            geos = {}
            for line in members['geography'].splitlines():
                if year == '2020':
                    fields = line.split(b'|')
                    if fields[2] != b'750':
                        continue
                    key, geoid = fields[7], fields[8].removeprefix(b'7500000US')
                    population = None
                else:
                    if line[8:11] != b'750':
                        continue
                    if year == '2000':
                        geoid = line[29:34] + line[55:61] + line[62:66]
                        key, population = geoid, int(line[292:301])
                    else:
                        key = line[18:25].strip()
                        geoid = line[27:29] + line[29:32] + line[54:60] + line[61:65]
                        population = None
                assert len(geoid) == 15 and geoid.isdigit() and key not in geos
                geos[key] = (geoid, population)
            assert len({v[0] for v in geos.values()}) == len(geos) == case['blocks']
            if year == '2000':
                pops = {key: value for key, (_, value) in geos.items()}
            else:
                pops = {}
                sep = b'|' if year == '2020' else b','
                for line in members['population'].splitlines():
                    fields = line.split(sep, 6)
                    if len(fields) < 6:
                        continue
                    key = fields[4].strip()
                    if key in geos:
                        assert key not in pops
                        pops[key] = int(fields[5])
            assert set(pops) == set(geos) and all(v >= 0 for v in pops.values())
            assert sum(pops.values()) == int(case['population'])
            results[scope] = {'crc_verified_members': names, 'exact_selected_members': list(members),
                              'blocks': len(geos), 'population': sum(pops.values())}
    assert set(results) == {'VT:2000', 'WA:2010', 'RI:2020'}
    return {'status': 'passed-independent-live-census-vintage-readback', 'results': results}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('downloads', type=Path)
    parser.add_argument('browser', type=Path)
    parser.add_argument('receipt', type=Path)
    args = parser.parse_args()
    result = audit(args.downloads, args.browser)
    with args.receipt.open('x', encoding='utf8') as output:
        json.dump(result, output, indent=2)
