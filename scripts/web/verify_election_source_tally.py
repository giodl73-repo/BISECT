"""Independently tally original raw votes from the frozen source-format proof."""
import argparse
import base64
import csv
import hashlib
import io
import json
import time
from pathlib import Path
from evidence_blob_store import ROOT, SnapshotBlobs, digest

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', default='target/election-source-format/snapshot-scope/snapshot.json')
    parser.add_argument('--sha256', default='7576fe2c0bf9d0daee69da217e3dacffe2afa495f139560c6c4401257cd70269')
    args = parser.parse_args()
    snapshot = SnapshotBlobs(args.snapshot, args.sha256)
    names = {r['path']: r for r in snapshot.data['files']}
    used = []
    def read(name):
        ref = names[name]
        used.append({k: ref[k] for k in ['path', 'sha256', 'byte_length']})
        return snapshot.blob(ref['sha256'], ref['byte_length']).read_bytes()
    records = []
    for profile in ['default', 'c-sidecar']:
        prefix = 'target/election-source-format/package-1793030387676235200/' + profile + '-unit/'
        project = json.loads(read(prefix + 'fixture.bisect'))
        original = base64.b64decode(project['files']['electionAnalysisSource']['base64'], validate=True)
        rows = list(csv.DictReader(io.StringIO(original.decode('utf-8-sig'), newline='')))
        plan = json.loads(base64.b64decode(project['files']['auditPlanText']['base64'], validate=True))['plan']
        assignment = dict(zip(plan['units']['unit_ids'], plan['assignment']))
        totals = {i: [0., 0.] for i in range(plan['k'])}
        output = io.StringIO(newline='')
        writer = csv.writer(output, lineterminator='\n')
        writer.writerow(['geoid', 'dem_votes', 'rep_votes'])
        for row in rows:
            unit = row['tract_GEOID'].strip().zfill(11)
            dem, rep = row['G20PREDBID'].strip(), row['G20PRERTRU'].strip()
            writer.writerow([unit, dem, rep])
            if unit in assignment:
                totals[assignment[unit]][0] += float(dem)
                totals[assignment[unit]][1] += float(rep)
        assert output.getvalue().encode() == read(prefix + 'canonical.csv')
        result = json.loads(read(prefix + 'result.json'))
        assert len(result['political']['districts']) == plan['k']
        for district in result['political']['districts']:
            assert [district['dem_votes'], district['rep_votes']] == totals[district['district'] - 1]
        bundle = json.loads(read(prefix + 'source-package.json'))
        assert base64.b64decode(bundle['source']['base64'], validate=True) == original
        assert base64.b64decode(bundle['canonical']['base64'], validate=True) == output.getvalue().encode()
        assert bundle['source']['sha256'] == hashlib.sha256(original).hexdigest()
        assert bundle['canonical']['sha256'] == hashlib.sha256(output.getvalue().encode()).hexdigest()
        records.append(dict(profile=profile, district_totals=totals, source_rows=len(rows),
                            matched_rows=len(assignment), extra_rows=len(rows) - len(assignment)))
    root = ROOT / 'target/election-source-format' / ('frozen-independent-tally-' + str(time.time_ns()))
    root.mkdir()
    script = Path(__file__).resolve()
    record = dict(status='passed', snapshot=dict(path=args.snapshot, sha256=snapshot.identity),
                  script=dict(path=script.relative_to(ROOT).as_posix(), sha256=digest(script), byte_length=script.stat().st_size),
                  input_files=used, records=records, native_analyzer_reused=False,
                  scope='Independent Python CSV parsing and direct per-district float aggregation from original frozen source/plan. Synthetic quarter/half vote fractions are exactly representable; no native analyzer or WASM reused. Not actual provider validation.')
    (root / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(root.relative_to(ROOT).as_posix() + '/evidence.json')

if __name__ == '__main__':
    main()
