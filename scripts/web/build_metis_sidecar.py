"""Build the explicit msvcrt-v1 C METIS module; never installs/downloads an SDK."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from probe_metis_c_sidecar import ROOT, identity, sources

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--emsdk',default='target/toolchains/emsdk')
    parser.add_argument('--output',default='target/cffi-integration/sidecar')
    args=parser.parse_args()
    sdk=(ROOT/args.emsdk).resolve();output=(ROOT/args.output).resolve()
    if not output.is_relative_to((ROOT/'target').resolve()):raise SystemExit('Build output must remain under repository target/')
    output.mkdir(parents=True,exist_ok=True)
    source,archive,c_sources=sources()
    owned=[ROOT/'crates/bisect-metis-sidecar/c'/name for name in ('bridge.c','msvcrt_rng.c')]
    settings=['-DIDXTYPEWIDTH=32','-DREALTYPEWIDTH=32','-DNDEBUG','-DNDEBUG2',
        '-Drand=bisect_msvcrt_rand','-Dsrand=bisect_msvcrt_srand','-O2',
        '-sMODULARIZE=1','-sEXPORT_ES6=1','-sENVIRONMENT=web,worker',
        '-sALLOW_MEMORY_GROWTH=1','-sFILESYSTEM=0','-sDYNAMIC_EXECUTION=0',
        '-sABORTING_MALLOC=0','-sEXPORTED_RUNTIME_METHODS=HEAPU8',
        '-sEXPORTED_FUNCTIONS=["_bisect_metis_partition","_malloc","_free"]']
    command=[sys.executable,str(sdk/'upstream/emscripten/emcc.py')]+[str(p) for p in c_sources+owned]+[
        '-I'+str(source/'vendor/GKlib'),'-I'+str(source/'vendor/metis/include'),
        '-I'+str(source/'vendor/metis/libmetis')]+settings+['-o',str(output/'metis-sidecar.mjs')]
    result=subprocess.run(command,cwd=ROOT,env=dict(os.environ,EM_CONFIG=str(sdk/'.emscripten')),
        capture_output=True,creationflags=0x08000000 if sys.platform=='win32' else 0)
    (output/'build.log').write_bytes(result.stdout+b'\n--- stderr ---\n'+result.stderr)
    if result.returncode:
        print(result.stderr.decode('utf-8',errors='replace'));return result.returncode
    record={'schema':'bisect-c-metis-sidecar-v1','runtime':'msvcrt-v1','abi':1,
        'javascript':identity(output/'metis-sidecar.mjs'),'wasm':identity(output/'metis-sidecar.wasm'),
        'source_archive':identity(archive),'source_files':[identity(p) for p in owned],
        'builder':identity(Path(__file__)),'vendor_verifier':identity(ROOT/'scripts/web/probe_metis_c_sidecar.py'),
        'compiler':identity(sdk/'upstream/emscripten/emcc.py'),
        'compiler_binary':identity(sdk/('upstream/bin/clang.exe' if sys.platform=='win32' else 'upstream/bin/clang')),
        'sdk_configuration':identity(sdk/'.emscripten'),
        'compiler_version':subprocess.check_output([sys.executable,str(sdk/'upstream/emscripten/emcc.py'),'--version'],text=True),
        'sdk_revision':subprocess.check_output(['git','-C',str(sdk),'rev-parse','HEAD'],text=True).strip(),
        'settings':settings,'arguments':command,'build_log':identity(output/'build.log'),
        'scope':'Serial C METIS sidecar with explicit Windows CRT RNG profile; no cross-platform native libc equivalence claim.'}
    (output/'metis-sidecar.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8',newline='\n')
    print('Built C METIS msvcrt-v1 sidecar:',record['wasm']['byte_length'],'WASM bytes')
    return 0

if __name__=='__main__':raise SystemExit(main())
