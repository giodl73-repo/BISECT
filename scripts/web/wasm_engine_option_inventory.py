"""Track engine-facing declarations; declaration coverage is not runtime parity."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/specs/wasm-engine-option-inventory.json'
SOURCES = [
    ('cli.run', 'crates/bisect-runner/src/run_args.rs', 'RunArgs'),
    ('cli.state', 'crates/bisect-runner/src/state_args.rs', 'StateArgs'),
    ('cli.states', 'crates/bisect-runner/src/states_args.rs', 'StatesArgs'),
    ('yaml', 'crates/bisect-runner/src/config.rs', 'AlgoYaml'),
    ('yaml.algorithm', 'crates/bisect-runner/src/config.rs', 'AlgorithmSection'),
    ('wasm.graph', 'crates/bisect-wasm/src/engine.rs', 'PreparedGraph'),
    ('wasm.options', 'crates/bisect-wasm/src/engine.rs', 'Options'),
]

def declarations():
    result = []
    for family, filename, struct in SOURCES:
        text = (ROOT / filename).read_text(encoding='utf-8')
        match = re.search(r'^pub struct '+struct+r'\s*\{(.*?)^\}', text, re.M | re.S)
        if not match:
            raise ValueError(f'Missing {filename}:{struct}')
        body = match.group(1)
        fields = list(re.finditer(r'^[ \t]*pub ([a-z][a-z0-9_]*):[ \t]*([^\r\n]+?),[ \t]*(?://([^\r\n]*))?$', body, re.M))
        # Fail closed if declarations stop fitting this explicitly scoped scanner.
        count = len(re.findall(r'\bpub\s+[a-z][a-z0-9_]*\s*:', body))
        if not fields or len(fields) != count:
            raise ValueError(f'Unsupported or missing field declaration in {struct}')
        previous_end = 0
        for field in fields:
            preceding = body[previous_end:field.start()].strip()
            result.append(dict(id=family+'.'+field[1], source=filename, struct=struct,
                               field=field[1], rust_type=field[2].strip(),
                               documentation_and_attributes=preceding,
                               trailing_comment=(field[3] or '').strip(),
                               review='unreviewed', evidence=[], limitation='Runtime behavior and UI parity are not established by this declaration.'))
            previous_end = field.end()
    return result

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    current = declarations()
    old = json.loads(OUT.read_text(encoding='utf-8')) if OUT.exists() else {'entries': []}
    if OUT.exists() and old.get('schema_version') != 1:
        raise SystemExit('Unsupported engine option inventory schema; review an explicit migration before regeneration.')
    previous = {entry['id']: entry for entry in old['entries']}
    keys = ['id', 'source', 'struct', 'field', 'rust_type', 'documentation_and_attributes', 'trailing_comment']
    if args.check:
        if set(previous) != {entry['id'] for entry in current}:
            raise SystemExit('Engine option declaration inventory has drifted; regenerate and review changes.')
        for entry in current:
            if any(entry[key] != previous[entry['id']][key] for key in keys):
                raise SystemExit('Engine option declaration changed: '+entry['id'])
    else:
        for entry in current:
            prior = previous.get(entry['id'])
            if prior:
                # Declaration identity changes never erase original review/history.
                # Existing source pins still require explicit reconciliation.
                for key, value in prior.items():
                    if key not in keys:
                        entry[key] = value
        output = dict(schema_version=1, scope='CLI Run/State/States flags, native YAML fields and browser graph/options declarations. Exact source drift guard; not a proof of execution or full parity.', entries=current)
        if 'parity_ledger' in old:
            output['parity_ledger'] = old['parity_ledger']
        for key, value in old.items():
            if key not in output:
                output[key] = value
        OUT.write_bytes((json.dumps(output, ensure_ascii=False, indent=2)+'\n').encode('utf-8'))
    counts = {family: sum(e['id'].startswith(family+'.') and e['struct']==struct for e in current) for family, _, struct in SOURCES}
    print(f'{len(current)} engine-facing fields match current sources: '+json.dumps(counts))

if __name__ == '__main__':
    main()
