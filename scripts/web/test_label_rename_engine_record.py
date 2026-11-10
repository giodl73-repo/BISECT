"""Adversarial reuse checks against a real successfully built engine record."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import time
from evidence_blob_store import ROOT, local

record=local(sys.argv[1]);source=json.loads(record.read_text(encoding='utf-8'))
root=ROOT/'target/label-rename-application'/('engine-reuse-validation-'+str(time.time_ns()));root.mkdir()
helper=ROOT/'scripts/web/validate_label_rename_engine_record.py'
r=subprocess.run([sys.executable,str(helper),'--record',str(record)],capture_output=True,text=True)
(root/'valid.log').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
cases=[]
def reject(name,mutate):
    data=copy.deepcopy(source);mutate(data);p=root/(name+'.json');p.write_text(json.dumps(data,indent=2),encoding='utf-8')
    reply=subprocess.run([sys.executable,str(helper),'--record',str(p)],capture_output=True,text=True)
    (root/(name+'.log')).write_text(reply.stdout+reply.stderr,encoding='utf-8');assert reply.returncode!=0,name
    cases.append(name)
reject('stale-source',lambda d:d['source_files'][0].update(sha256='0'*64))
reject('wrong-profile',lambda d:d['profiles'].update({'c-sidecar':copy.deepcopy(d['profiles']['default'])}))
reject('failed-prerequisite',lambda d:d['stages'][0].update(exit_code=1))
reject('changed-log',lambda d:d['stages'][0].update(sha256='0'*64))
reject('missing-native-safety',lambda d:d.update(stages=[s for s in d['stages'] if s['name']!='native-case-alias-cli']))
reject('wrong-build-flags',lambda d:next(s for s in d['stages'] if s['name']=='c-sidecar-build').update(arguments=['cargo','build']))
if 'native_summary_reference' in source:
    reject('changed-summary-reference',lambda d:d['native_summary_reference'].update(sha256='0'*64))
    reject('missing-summary-clock-tests',lambda d:d.update(stages=[s for s in d['stages'] if s['name']!='native-report-clock-tests']))
    reject('missing-summary-browser',lambda d:d['profiles']['default'].pop('application_browser'))
escape=root/'escaped.wasm';reply=subprocess.run([sys.executable,str(helper),'--record',str(record),'--profile','default','--destination',str(escape)],capture_output=True,text=True);(root/'escaped-destination.log').write_text(reply.stdout+reply.stderr,encoding='utf-8');assert reply.returncode!=0 and not escape.exists();cases.append('escaped-destination')
(root/'evidence.json').write_text(json.dumps(dict(status='passed',valid_real_record=True,rejected_cases=cases,scope='Real supplied engine evidence; exact source/runtime/log/profile/prerequisite and owned-destination refusal. No record-authenticity claim.'),indent=2),encoding='utf-8')
print(json.dumps(dict(root=str(root),rejected_cases=len(cases))))
