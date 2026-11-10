"""Check downloaded custom-jurisdiction archives without extracting filesystem paths."""
from pathlib import Path
import hashlib,json,zipfile
roots=[Path('target/custom-jurisdiction-engine'),Path('target/custom-jurisdiction-browser')]
count=files=0
for root in roots:
    for archive in sorted(root.glob('*-sources.zip')):
        state=archive.name[:2]
        with zipfile.ZipFile(archive) as z:
            assert z.testzip() is None
            assert len(z.namelist())==len(set(z.namelist()))
            manifest=json.loads(z.read('bundle.json'))
            assert set(z.namelist())==set(manifest['files'])|{'bundle.json'}
            for name,record in manifest['files'].items():
                assert not name.startswith('/') and all(p not in ('','..','.') for p in name.split('/'))
                data=z.read(name)
                assert record['bytes']==len(data)
                assert record['sha256']==hashlib.sha256(data).hexdigest()
                files+=1
            graph=json.loads(z.read('sources/graph.json'))
            plan=json.loads(z.read('plan.rplan'))
            context=json.loads(z.read('context.rctx'))
            assert graph['state']==plan['plan']['units']['state']==context['units']['state']==state
            assert str(plan['plan']['units']['year'])==str(context['units']['year'])==graph['year']=='2024'
            assert plan['plan']['units']['unit_kind']==context['units']['unit_kind']==graph['unit_kind']
            assert plan['plan']['units']['unit_ids']==context['units']['unit_ids']==graph['geoids']
            assert context['subdivisions']['county_ids']==graph['county_ids']
            count+=1
assert count==6, f'Run engine and browser gates first; found {count} archives'
Path('target/custom-jurisdiction-engine/zip-evidence.json').write_text(json.dumps({'archives':count,'files_hashed':files,'crc_paths_hashes_scope_unit_and_county_order':True},indent=2)+'\n')
print(f'PASS custom jurisdiction ZIPs: {count} archives / {files} hashed files, exact scope and unit/county order')
