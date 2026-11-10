"""Independent ZIP/CRC, native assignment and exact u64 manifest checks."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('roots', nargs='+')
    parser.add_argument('--native-reference', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    native_root = Path(args.native_reference)
    native = json.loads((native_root / 'evidence.json').read_text(encoding='utf-8'))
    assert native['status'] == 'passed'
    outcomes = []
    for folder in args.roots:
        root = Path(folder)
        for row in native['outcomes']:
            if row['name'] == 'registry-default':
                continue
            manifest_ref = next(r for r in row['files'] if r['path'].endswith('/manifest.json'))
            native_folder = Path(manifest_ref['path']).parent
            manifest = json.loads(Path(manifest_ref['path']).read_text(encoding='utf-8'))
            relative = native_folder.resolve().relative_to((native_root / row['name']).resolve()).as_posix()
            namespace = 'v1/' + relative + '/'
            archive = root / (row['name'] + '.zip')
            with zipfile.ZipFile(archive) as z:
                assert z.testzip() is None
                assert all(name.startswith(namespace) for name in z.namelist())
                actual = json.loads(z.read(namespace + 'plan.rplan'))
                expected = json.loads((native_folder / 'plan.rplan').read_text(encoding='utf-8'))
                assert actual['plan']['assignment'] == expected['plan']['assignment']
                assert actual['plan']['units']['unit_ids'] == expected['plan']['units']['unit_ids']
                report = json.loads(z.read(namespace + 'manifest.json'))
                assert report['engine_seed'] == 2**64 - 1
                assert report['label'] == manifest['label']
                assert report['adjacency_file'] == manifest['adjacency_file']
                assert report['adjacency_sha256'] == manifest['adjacency_sha256']
                assert report['source_tool'] == 'bisect-wasm'
                assert json.loads(z.read(namespace + 'data/final_assignments.json')) == json.loads((native_folder / 'data/final_assignments.json').read_text(encoding='utf-8'))
                assert namespace + relative.rsplit('/', 1)[1] + '.rplan' in z.namelist() if '/states/' in namespace else namespace + manifest['label'] + '.rplan' in z.namelist()
            outcomes.append(dict(path=archive.as_posix(), sha256=hashlib.sha256(archive.read_bytes()).hexdigest(), namespace=namespace, exact_native_assignments=True, exact_unsigned_seed=True, zip_crc=True))
    output = Path(args.output)
    assert not output.exists()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(dict(status='passed',outcomes=outcomes,scope='Independent stdlib ZIP parsing/CRC and Python exact integers. Only named-source manifest fields/assignments are compared; complete native executable/provenance equivalence is not claimed.'), indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(status='passed',archives=len(outcomes),evidence=str(output))))


if __name__ == '__main__':
    main()
