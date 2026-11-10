"""Copy the native TUI glossary literally; --check detects source drift."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
source = root / 'crates/bisect-tui/src/widgets/glossary.rs'
text = source.read_text(encoding='utf-8').split('pub const GLOSSARY_ENTRIES:', 1)[1].split('];', 1)[0]
strings = re.findall(r'"(?:[^"\\]|\\.)*"', text)
assert len(strings) == 14, 'Review native glossary syntax/entry changes before publishing.'
entries = [list(map(json.loads, strings[i:i+2])) for i in range(0, len(strings), 2)]
output = '// Generated from crates/bisect-tui/src/widgets/glossary.rs by scripts/web/native_metric_glossary.py.\nexport const NATIVE_METRIC_GLOSSARY=Object.freeze(' + json.dumps(entries, ensure_ascii=False, indent=2) + '.map(entry=>Object.freeze(entry)));\n'
target = root / 'web/lab/native-metric-glossary.js'
if '--check' in sys.argv:
    assert target.read_text(encoding='utf-8') == output, 'Native/browser glossary drift; regenerate.'
else:
    target.write_bytes(output.encode('utf-8'))
print('All seven native glossary terms and definitions match.')
