"""Generate the browser refusal notice from the actual prepared State guard."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
source = (root / 'crates/bisect-wasm/src/native_state_run.rs').read_text(encoding='utf-8')
fields = json.loads('[' + re.search(r'let unsupported=\[([^\]]+)\]', source).group(1) + ']')
inventory = json.loads((root / 'docs/specs/wasm-engine-option-inventory.json').read_text(encoding='utf-8'))
assert len(fields) == len(set(fields))
assert set(fields) <= {row['field'] for row in inventory['entries'] if row['id'].startswith('cli.state.')}
text = '// Generated from native_state_run.rs; check with scripts/web/native_state_boundary.py --check.\n'
text += 'export const NATIVE_STATE_PREPARED_REFUSED_FIELDS=Object.freeze(' + json.dumps(fields, indent=2) + ');\n'
booleans = [row['field'] for row in inventory['entries'] if row['id'].startswith('cli.state.') and row['rust_type'] == 'bool']
text += 'export const NATIVE_STATE_BOOLEAN_FIELDS=Object.freeze(' + json.dumps(booleans, indent=2) + ');\n'
path = root / 'web/lab/native-state-boundary.js'
if '--check' in sys.argv:
    assert path.read_text(encoding='utf-8') == text, 'Prepared State browser refusal notice drifted from native compiler'
else:
    path.write_text(text, encoding='utf-8')
print(f'{len(fields)} prepared State refusal fields match the actual compiler')
