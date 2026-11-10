"""Generate browser analyzer names/defaults from native code; --check detects drift."""
from pathlib import Path
import json,re,sys
root=Path(__file__).resolve().parents[2]
text=(root/'crates/bisect-analysis/src/analyzer.rs').read_text(encoding='utf-8')
names=dict(re.findall(r'Self::(\w+)\s*=>\s*"([a-z-]+)"',text))
body=text.split('pub fn all_concrete()',1)[1].split('vec![',1)[1].split(']',1)[0]
defaults=[names[name] for name in re.findall(r'Self::(\w+)',body)]
concrete=[name for variant,name in names.items() if variant!='All']
assert len(concrete)==10 and len(defaults)==7,'Review native analyzer catalog change.'
output='// Generated from native AnalyzerType by scripts/web/native_aggregate_catalog.py.\nexport const NATIVE_AGGREGATE_TYPES=Object.freeze('+json.dumps(concrete)+');\nexport const NATIVE_AGGREGATE_ALL=Object.freeze('+json.dumps(defaults)+');\n'
target=root/'web/lab/native-aggregate-catalog.js'
if '--check' in sys.argv:assert target.read_text(encoding='utf-8')==output,'Native/browser analyzer catalog drift.'
else:target.write_bytes(output.encode('utf-8'))
print('Native aggregate catalog: ten concrete types and seven defaults match.')
