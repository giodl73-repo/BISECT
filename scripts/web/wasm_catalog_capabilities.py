"""Use the current native WASM exporter declarations when refreshing a staged catalog."""
from pathlib import Path
import json
import re

ROOT=Path(__file__).resolve().parents[2]
SOURCE=ROOT/'crates/bisect-web/src/bin/wasm-lab-assets.rs'

def capabilities():
    text=SOURCE.read_text(encoding='utf-8')
    def values(pattern):
        match=re.search(pattern,text)
        if not match:raise ValueError('Native exporter capability declaration changed')
        source=match.group(1)
        items=re.findall(r'"([a-z][a-z-]*)"',source)
        if not items or len(set(items))!=len(items):raise ValueError('Invalid native capability declaration')
        return items
    structures=values(r'let structures=\[(.*?)\];')
    searches=values(r'let standard=\[(.*?)\];')
    weights=values(r'\["weights"\]=json!\(\[(.*?)\]\)')
    multi=values(r'else if \[(.*?)\]\.contains\(s\)')
    return dict(structures=structures,searches=searches,weights=weights,search_compatibility={s:searches if s=='standard-bisect' else ['single','multi'] if s in multi else ['single'] for s in structures})

if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser();parser.add_argument('catalog',type=Path);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    data=json.loads(args.catalog.read_text(encoding='utf-8'));current=capabilities()
    if args.check:
        for key,value in current.items():
            if data['catalog'].get(key)!=value:raise SystemExit('Staged native WASM capability drift: '+key)
    else:
        data['catalog'].update(current);args.catalog.write_bytes((json.dumps(data,indent=2)+'\n').encode('utf-8'))
    print(f'Native WASM exporter capabilities: {len(current["structures"])} structures, {len(current["searches"])} searches, {len(current["weights"])} weights match.')
