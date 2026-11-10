"""Validate this slice's owned engine evidence without accepting a package."""
import argparse
import json
import shutil
from evidence_blob_store import ROOT,local,digest


def validate(record):
    record=local(record)
    data=json.loads(record.read_text(encoding='utf-8'))
    assert data['status']=='passed' and data['full_parity'] is False
    def check(ref):
        p=local(ref['path'])
        assert digest(p)==ref['sha256'] and p.stat().st_size==ref['byte_length'], 'Changed engine evidence: '+str(p)
        return p
    assert data['source_files']
    for ref in [*data['source_files'],data['native_cli'],data['native_host']]:check(ref)
    stages={s['name']:s for s in data['stages']}
    assert len(stages)==len(data['stages'])
    required=['native-cli','native-host','default-build','default-application','c-sidecar-build','c-sidecar-application']
    for name in required:assert name in stages and stages[name]['exit_code']==0, 'Missing passed engine stage: '+name
    for s in data['stages']:
        assert s['exit_code']==0
        check(s['log'])
    assert stages['native-cli']['args']==['cargo','build','-p','bisect-cli','--bin','bisect','--release','--no-default-features','--locked','--offline','-j','1']
    assert stages['native-host']['args']==['cargo','build','-p','bisect-wasm','--release','--example','execute_tool_request','--locked','--offline','-j','1']
    baseline=['cargo','build','-p','bisect-wasm','--release','--target','wasm32-unknown-unknown','--locked','--offline','-j','1']
    assert stages['default-build']['args']==baseline
    assert stages['c-sidecar-build']['args']==baseline+['--features','c-metis-sidecar']
    for profile in ['default','c-sidecar']:
        item=data['profiles'][profile];artifact=item['artifact'];check(artifact)
        assert stages[profile+'-application']['args']==['node','scripts/web/test_label_build_application.mjs']
        recorded=json.loads(local(stages[profile+'-application']['log']['path']).read_text(encoding='utf-8'))
        assert recorded==item['application'], 'Application result/log substitution'
        direct=json.loads((local(item['application']['root'])/'evidence.json').read_text(encoding='utf-8'))
        assert direct['runtime']['kind']=='wasm' and local(direct['runtime']['path'])==local(artifact['path'])
        assert direct['runtime']['sha256']==artifact['sha256']
        assert direct['native']['sha256']==data['native_cli']['sha256'] and local(direct['native']['path'])==local(data['native_cli']['path'])
        assert {r['name'] for r in direct['outcomes']}=={'normal','force','skip','dry-force','mixed-years','partial','empty-registry'}
        assert all(r['native_exit']==0 for r in direct['outcomes']) and len(direct['recovery'])==22
        transport=json.loads((local(item['application']['root'])/'progress-transport-evidence.json').read_text(encoding='utf-8'))
        assert {r['mode'] for r in transport['cases']}=={'cancel-before-reply','reply-success','wrong-state','changed-final'}
    return data,dict(path=record.relative_to(ROOT).as_posix(),sha256=digest(record),byte_length=record.stat().st_size)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record',required=True)
    parser.add_argument('--profile',choices=['default','c-sidecar'])
    parser.add_argument('--destination')
    args=parser.parse_args();data,reference=validate(args.record)
    assert bool(args.profile)==bool(args.destination)
    if args.profile:
        target=local(args.destination)
        assert target==ROOT/'target/cffi-legacy-gates'/(args.profile+'.wasm'), 'Copy escaped owned gate target'
        shutil.copy2(local(data['profiles'][args.profile]['artifact']['path']),target)
        assert digest(target)==data['profiles'][args.profile]['artifact']['sha256']
    print(json.dumps(dict(mode='verified-build-application-engine-record',record=reference,profile=args.profile,source_pins_current=True,recompiled_in_this_runner=False,package_accepted=False)))


if __name__=='__main__':main()
