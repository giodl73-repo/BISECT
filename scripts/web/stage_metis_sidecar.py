"""Stage verified C assets into a local toolkit; does not build or deploy."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import tarfile

from probe_metis_c_sidecar import METIS_SYS_CHECKSUM

ROOT=Path(__file__).resolve().parents[2]
NOTICES={
    'LICENSE-METIS.txt': 'metis-sys-0.3.2/vendor/metis/LICENSE',
    'LICENSE-GKlib.txt': 'metis-sys-0.3.2/vendor/GKlib/LICENSE.txt',
}
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def vendor_notices(receipt):
    # Use the same checksum-pinned crate that supplied the C sources, not
    # a license from a newer upstream release or an unverified extracted tree.
    if receipt['sha256']!=METIS_SYS_CHECKSUM:raise SystemExit('Unknown METIS vendor archive')
    cargo=Path(os.environ.get('CARGO_HOME',Path.home()/'.cargo'))
    archive=next((p for p in sorted((cargo/'registry/cache').glob('*/metis-sys-0.3.2.crate'))
                  if p.stat().st_size==receipt['byte_length'] and sha(p)==METIS_SYS_CHECKSUM),None)
    if archive is None:raise SystemExit('Require the pinned cached METIS vendor archive to stage its notices')
    with tarfile.open(archive,'r:gz') as package:
        result={}
        for name,member_path in NOTICES.items():
            member=package.getmember(member_path)
            if not member.isfile() or not 0<member.size<64*1024:raise SystemExit('Invalid vendor notice: '+member_path)
            with package.extractfile(member) as f:result[name]=f.read()
        return result

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source',default='target/cffi-integration/sidecar')
    p.add_argument('--destination',required=True)
    a=p.parse_args();source=(ROOT/a.source).resolve();destination=(ROOT/a.destination).resolve()
    if not any(destination.is_relative_to(ROOT/name) for name in ('target','dist')):raise SystemExit('Select a local target/ or dist/ toolkit')
    if not (destination/'index.html').is_file() or not (destination/'bisect_wasm.wasm').is_file():raise SystemExit('Stage an existing toolkit engine first')
    manifest=json.loads((source/'metis-sidecar.json').read_text(encoding='utf-8'))
    if manifest.get('schema')!='bisect-c-metis-sidecar-v1' or manifest.get('runtime')!='msvcrt-v1':raise SystemExit('Unknown sidecar contract')
    for name,key in [('metis-sidecar.mjs','javascript'),('metis-sidecar.wasm','wasm')]:
        path=source/name
        if sha(path)!=manifest[key]['sha256'] or path.stat().st_size!=manifest[key]['byte_length']:raise SystemExit('Stale sidecar asset: '+name)
    for ref in manifest['source_files']+[manifest['builder'],manifest['vendor_verifier']]:
        path=(ROOT/ref['path']).resolve()
        if not path.is_relative_to(ROOT) or sha(path)!=ref['sha256']:raise SystemExit('Stale sidecar source')
    notices=vendor_notices(manifest['source_archive'])
    output=destination/'c-metis';output.mkdir(parents=True,exist_ok=True)
    for name in ['metis-sidecar.mjs','metis-sidecar.wasm']:shutil.copy2(source/name,output/name)
    for name,data in notices.items():(output/name).write_bytes(data)
    # Public runtime metadata names the assets, not developer-local cache paths.
    public={'schema':manifest['schema'],'runtime':manifest['runtime'],'abi':manifest['abi'],
            'source_archive_sha256':manifest['source_archive']['sha256'],
            'build_receipt_sha256':sha(source/'metis-sidecar.json')}
    for key,name in [('javascript','metis-sidecar.mjs'),('wasm','metis-sidecar.wasm')]:
        public[key]={'file':name,'sha256':manifest[key]['sha256'],'byte_length':manifest[key]['byte_length']}
    (output/'metis-sidecar.json').write_text(json.dumps(public,indent=2)+'\n',encoding='utf-8',newline='\n')
    print('Staged verified msvcrt-v1 C METIS assets:',output.relative_to(ROOT))
if __name__=='__main__':main()
