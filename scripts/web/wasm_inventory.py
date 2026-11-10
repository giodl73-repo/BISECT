"""Record command coverage without treating compilation as implementation."""
import json, pathlib, re, sys
root = pathlib.Path(__file__).resolve().parents[2]
sources = [
    ('bisect', 'crates/bisect-cli/src/args.rs', 'Commands'),
    ('suite', 'crates/bisect-cli/src/args.rs', 'SuiteCommands'),
    ('config', 'crates/bisect-cli/src/args.rs', 'ConfigCommands'),
    ('ceremony', 'crates/bisect-cli/src/ceremony_cmd.rs', 'CeremonySubcommand'),
    ('civic', 'crates/bisect-cli/src/civic.rs', 'CivicSubcommand'),
    ('research', 'crates/bisect-cli/src/research.rs', 'ResearchSubcommand'),
    ('rplan', 'crates/rplan-cli/src/main.rs', 'Commands'),
    ('rcount', 'crates/rcount-cli/src/main.rs', 'Commands'),
    ('rhist', 'crates/rhist-cli/src/main.rs', 'Commands'),
    ('tui', 'crates/bisect-tui/src/app.rs', 'Screen'),
]
entries = []
output = root / 'docs/specs/wasm-capability-inventory.json'
old = json.loads(output.read_text(encoding='utf-8')) if output.exists() else {'entries': []}
if output.exists() and old.get('schema_version') != 1:
    raise ValueError('Unsupported command inventory schema; review an explicit migration before regeneration.')
previous = {item['id']: item for item in old['entries']}
for family, source, enum in sources:
    text = (root / source).read_text(encoding='utf-8')
    match = re.search(r'\benum\s+' + enum + r'\s*\{(.*?)^\}', text, re.M | re.S)
    if not match: raise ValueError(f'Missing command enumeration {source}:{enum}')
    variants = re.findall(r'^    ([A-Z][A-Za-z0-9_]*)(?:\(|\s*\{|,)', match[1], re.M)
    if not variants: raise ValueError(f'Empty command enumeration {source}:{enum}')
    for command in variants:
        entry=dict(id=f'{family}.{command}', source=source, enum=enum, operation='pending', ui='pending', verification='pending')
        prior=previous.get(entry['id'], {})
        # Current declaration identity wins; every other field is historical data.
        combined={**prior, **{key:entry[key] for key in ('id','source','enum')}}
        for key in ('operation','ui','verification'): combined.setdefault(key,entry[key])
        entries.append(combined)
if '--check' in sys.argv:
    if {item['id'] for item in entries} != set(previous): raise SystemExit('Command coverage inventory has drifted; regenerate it.')
    for entry in entries:
        if any(entry[key] != previous[entry['id']].get(key) for key in ('source', 'enum')):
            raise SystemExit('Command declaration authority changed: ' + entry['id'])
    print(f'{len(entries)} command/screen entries match current sources')
    raise SystemExit(0)
result = {**old, 'schema_version':1, 'entries':entries}
output.write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
print(f'{len(entries)} command/screen entries recorded in {output}')
