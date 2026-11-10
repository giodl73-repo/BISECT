"""Pinned direct-read traces for Run/State/States; not execution evidence.

The reviewed unused sets describe the CLI entry branch plus its delegated
configuration constructor. They do not infer downstream effects from a read.
Inspector/debug rendering and tests are excluded from the scan.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

from wasm_parity_ledger import ROOT, pin

INVENTORY = 'docs/specs/wasm-engine-option-inventory.json'
MAIN = 'crates/bisect-cli/src/main.rs'
FAMILIES = {
    'cli.run': ('Run', 'crates/bisect-runner/src/run_command.rs', 'pub fn from_args(args:&RunArgs)',
                ['dpi', 'election_year', 'experiment_name', 'minimum_boundary_length', 'print_only',
                 'processing_mode', 'run_type', 'skip_analysis', 'skip_demographic', 'skip_political', 'stages']),
    'cli.state': ('State', 'crates/bisect-runner/src/state_config.rs', 'pub fn from_state_args(',
                  ['dpi', 'objtype', 'print_only', 'target_mm_districts']),
    'cli.states': ('States', 'crates/bisect-runner/src/states_config.rs', 'pub fn from_args(args:&StatesArgs)',
                   ['dpi', 'election_year', 'print_only', 'processing_mode', 'run_analysis',
                    'skip_demographic', 'skip_political']),
}


def code_mask(text):
    """Keep source offsets while hiding comments/strings from direct-read scans."""
    result = list(text)
    i = 0
    while i < len(text):
        start = i
        if text.startswith('//', i):
            end = text.find('\n', i)
            i = len(text) if end < 0 else end
        elif text.startswith('/*', i):
            depth = 1
            i += 2
            while depth and i < len(text):
                if text.startswith('/*', i):
                    depth += 1
                    i += 2
                elif text.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError('Unterminated Rust comment')
        else:
            raw = re.match(r'(?:br|r)(#{0,255})"', text[i:]) if text[i] in 'br' and (i == 0 or not (text[i-1].isalnum() or text[i-1] == '_')) else None
            if raw:
                delimiter = '"' + raw[1]
                end = text.find(delimiter, i + raw.end())
                if end < 0:
                    raise ValueError('Unterminated Rust raw string')
                i = end + len(delimiter)
            elif text[i] == '"':
                i += 1
                while i < len(text):
                    if text[i] == '\\':
                        i += 2
                    elif text[i] == '"':
                        i += 1
                        break
                    else:
                        i += 1
                else:
                    raise ValueError('Unterminated Rust string')
            elif text[i] == "'" and (character := re.match(r"'(?:\\[^\n]|[^'\\\n])'", text[i:])):
                i += len(character[0])
            else:
                i += 1
                continue
        for index in range(start, min(i, len(text))):
            if result[index] not in '\r\n':
                result[index] = ' '
    return ''.join(result)


def body(text, marker):
    masked = code_mask(text)
    start = masked.index(marker)
    brace = masked.index('{', start)
    depth = 0
    for end in range(brace, len(masked)):
        if masked[end] == '{':
            depth += 1
        elif masked[end] == '}':
            depth -= 1
            if depth == 0:
                return start, end + 1, masked[start:end+1]
    raise ValueError('Unbalanced native body: ' + marker)


def traces(root, entries):
    result = {}
    for family, (command, configuration, marker, expected_unused) in FAMILIES.items():
        fields = {e['field']: e for e in entries if e['id'].startswith(family + '.')}
        sites = {field: [] for field in fields}
        sources = []
        for source, locator in ((MAIN, 'Commands::' + command + '(args) =>'), (configuration, marker)):
            text = (root / source).read_text(encoding='utf-8')
            start, end, masked = body(text, locator)
            sources.append({**pin(root, source), 'locator': locator,
                            'body_sha256': hashlib.sha256(text[start:end].encode()).hexdigest()})
            for match in re.finditer(r'\bargs\s*\.\s*([a-z_][a-z0-9_]*)\b', masked):
                field = match[1]
                if field not in fields:
                    raise ValueError(f'{family}: unrecognized direct argument read {field}')
                sites[field].append({'path': source, 'line': text.count('\n', 0, start + match.start()) + 1})
        unused = sorted(field for field in fields if not sites[field])
        if unused != expected_unused:
            raise ValueError(f'{family}: reviewed unused set changed: {unused!r}')
        for field, entry in fields.items():
            result[entry['id']] = {
                'schema': 'bisect-native-direct-read-v1',
                'scope': 'CLI entry branch and delegated configuration constructor only; inspector, diagnostics formatter and tests excluded. A direct read is not proof of downstream algorithm or filesystem effect.',
                'sources': sources, 'direct_reads': sites[field],
                'finding': 'direct-read' if sites[field] else 'no-direct-read-in-reviewed-boundary',
                'runtime_evidence': False,
            }
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='Reconcile source traces only; never promotes runtime evidence')
    args = parser.parse_args()
    path = ROOT / INVENTORY
    data = json.loads(path.read_text(encoding='utf-8'))
    current = traces(ROOT, data['entries'])
    for entry in data['entries']:
        if entry['id'] not in current:
            continue
        row = entry['parity_record']
        if args.write:
            row['native_consumption'] = current[entry['id']]
        elif row.get('native_consumption') != current[entry['id']]:
            raise SystemExit('Native consumption trace stale or missing: ' + entry['id'])
    if args.write:
        path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(f'{len(current)} native field traces match reviewed source boundaries; 22 fields have no direct read. No execution claim.')


if __name__ == '__main__':
    main()
