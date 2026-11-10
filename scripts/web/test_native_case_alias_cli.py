"""Actual CLI case-only rename refusal, including preservation of every stage."""
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]

def main():
    root = ROOT/'target/label-rename-application'/('native-case-alias-cli-'+str(time.time_ns()))
    root.mkdir(parents=True)
    source_exe = Path(os.environ.get('BISECT_TEST_NATIVE_CLI', ROOT/'target/release/bisect.exe')).resolve()
    exe = root/'bisect.exe'
    shutil.copy2(source_exe, exe)
    cases = []
    for index, (source, destination) in enumerate([('alpha', 'Alpha'), ('ALPHA', 'alpha')]):
        for force in [False, True]:
            folder = root/(str(index)+'-'+source+'-'+str(force))
            folder.mkdir()
            original = {'.bisect': json.dumps({source:dict(built=['2020'],analyzed=[],reported=[])}).encode()}
            for stage in ['runs','analysis','reports']:
                original[f'{stage}/{source}/sentinel.bin'] = bytes([0,255,128,13,10])
            for name, value in original.items():
                path = folder/name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(value)
            arguments = ['mv',source,destination]+(['--force'] if force else [])
            result = subprocess.run([str(exe),*arguments],cwd=folder,capture_output=True,text=True)
            actual = {p.relative_to(folder).as_posix():p.read_bytes() for p in folder.rglob('*') if p.is_file()}
            record = dict(arguments=arguments,exit_code=result.returncode,stdout=result.stdout,stderr=result.stderr,
                          original={k:base64.b64encode(v).decode() for k,v in original.items()},
                          actual={k:base64.b64encode(v).decode() for k,v in actual.items()})
            cases.append(record)
            (root/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
            assert result.returncode!=0 and 'case-only' in result.stderr, record
            assert actual==original, 'Case-only rename changed source stages, registry or created extra files'
    (root/'evidence.json').write_text(json.dumps(dict(schema_version='bisect-native-case-alias-safety-v1',
        native_executable=dict(path=str(exe.relative_to(ROOT)).replace(chr(92),'/'),sha256=hashlib.sha256(exe.read_bytes()).hexdigest()),
        cases=len(cases),all_source_and_registry_bytes_preserved=True,no_files_created=True,
        scope='Actual CLI Windows witness plus portable refusal; no whole-rename transaction/lock claim.'),indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(root=str(root),cases=len(cases))))

if __name__=='__main__':
    main()
