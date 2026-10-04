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
previous = {item['id']: item for item in json.loads(output.read_text())['entries']} if output.exists() else {}
for family, source, enum in sources:
    text = (root / source).read_text(encoding='utf-8')
    match = re.search(r'\benum\s+' + enum + r'\s*\{(.*?)^\}', text, re.M | re.S)
    if not match: raise ValueError(f'Missing command enumeration {source}:{enum}')
    variants = re.findall(r'^    ([A-Z][A-Za-z0-9_]*)(?:\(|\s*\{|,)', match[1], re.M)
    if not variants: raise ValueError(f'Empty command enumeration {source}:{enum}')
    for command in variants:
        entry=dict(id=f'{family}.{command}', source=source, enum=enum, operation='pending', ui='pending', verification='pending')
        prior=previous.get(entry['id'], {})
        for key in ['operation','ui','verification','evidence']:
            if key in prior: entry[key]=prior[key]
        entries.append(entry)
if '--check' in sys.argv:
    if {item['id'] for item in entries} != set(previous): raise SystemExit('Command coverage inventory has drifted; regenerate it.')
    print(f'{len(entries)} command/screen entries match current sources')
    raise SystemExit(0)
output.write_text(json.dumps(dict(schema_version=1, entries=entries), indent=2)+'\n', encoding='utf-8')
print(f'{len(entries)} command/screen entries recorded in {output}')
