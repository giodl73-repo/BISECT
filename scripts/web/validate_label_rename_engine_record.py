"""Validate owned fresh engine evidence before reusing its exact bytes in regression gates."""
import argparse
import json
from pathlib import Path
import shutil
from evidence_blob_store import ROOT, local, digest


def validate(record):
    record = local(record)
    data = json.loads(record.read_text(encoding='utf-8'))
    assert data['status']=='passed' and data['full_parity'] is False
    assert data['source_files'] and all(s['exit_code']==0 for s in data['stages'])
    stages = {s['name']:s for s in data['stages']}
    for name in ['shared-rename-kernel','native-rename-tests','native-cli','native-case-alias-cli','native-host','default-build','default-rename','c-sidecar-build','c-sidecar-rename']:
        assert name in stages, 'Missing executed engine prerequisite: '+name
    for ref in [*data['source_files'],data['native_cli'],data['native_host'],*data['profiles'].values()]:
        path = local(ref['path'])
        assert digest(path)==ref['sha256'], 'Engine evidence drift: '+str(path)
        assert path.stat().st_size==ref['byte_length']
    for stage in data['stages']:
        assert digest(local(stage['log']))==stage['sha256'], 'Changed engine build/test log'
    assert 'crates/bisect-report/src/label_rename.rs' in {r['path'] for r in data['source_files']}
    assert stages['default-build']['arguments']==['cargo','build','-p','bisect-wasm','--release','--target','wasm32-unknown-unknown','--locked','--offline','-j','1']
    assert stages['c-sidecar-build']['arguments']==stages['default-build']['arguments']+['--features','c-metis-sidecar']
    for profile in ['default','c-sidecar']:
        runtime=data['profiles'][profile]
        for kind in ['direct','batch','rename']:
            assert local(stages[profile+'-'+kind]['wasm'])==local(runtime['path']), 'Profile/test runtime substitution'
            tested=json.loads((local(runtime[kind]['root'])/'evidence.json').read_text(encoding='utf-8'))
            assert tested['wasm']['sha256']==runtime['sha256'], 'Tested engine differs from supplied runtime'
        renamed=json.loads((local(runtime['rename']['root'])/'evidence.json').read_text(encoding='utf-8'))
        assert renamed['native_sha256']==data['native_cli']['sha256'], 'Native CLI differs from exact file comparisons'
    application = 'native_summary_reference' in data or any('application' in p for p in data['profiles'].values()) or 'crates/bisect-cli/examples/label_summary_application_reference.rs' in {r['path'] for r in data['source_files']}
    if application:
        assert 'native-report-clock-tests' in stages, 'Missing native clock regressions'
        reference = data['native_summary_reference']
        path = local(reference['path'])
        assert digest(path)==reference['sha256'] and path.stat().st_size==reference['byte_length'], 'Native summary reference drift'
        for profile in ['default','c-sidecar']:
            runtime=data['profiles'][profile]
            for kind,stage_name in [('application','summary-application'),('application_browser','summary-application-browser')]:
                assert local(stages[profile+'-'+stage_name]['wasm'])==local(runtime['path']), 'Summary application runtime substitution'
                tested=json.loads((local(runtime[kind]['root'])/'evidence.json').read_text(encoding='utf-8'))
                if kind=='application':
                    assert tested['wasm']['sha256']==runtime['sha256'] and tested['native']['sha256']==data['native_cli']['sha256']
                    assert tested['fixed_clock_native']['sha256']==reference['sha256'], 'Summary application native reference substitution'
                else:
                    assert tested['wasm_sha256']==runtime['sha256'] and tested['native_sha256']==reference['sha256'], 'Summary browser reference/runtime substitution'
    return data,dict(path=record.relative_to(ROOT).as_posix(),sha256=digest(record))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record',required=True)
    parser.add_argument('--profile',choices=['default','c-sidecar'])
    parser.add_argument('--destination')
    args=parser.parse_args()
    data,reference=validate(args.record)
    assert bool(args.profile)==bool(args.destination)
    if args.profile:
        target=local(args.destination)
        assert target==ROOT/'target/cffi-legacy-gates'/(args.profile+'.wasm'), 'Runtime copy escaped owned gate destination'
        shutil.copy2(local(data['profiles'][args.profile]['path']),target)
        assert digest(target)==data['profiles'][args.profile]['sha256']
    print(json.dumps(dict(mode='verified-owned-engine-record',record=reference,source_pins_current=True,profile=args.profile,recompiled_in_this_runner=False)))


if __name__=='__main__':
    main()
