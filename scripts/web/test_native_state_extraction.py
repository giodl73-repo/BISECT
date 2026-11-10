"""Verify original extraction bodies and explicitly reviewed source transitions.

The original baseline stays immutable. A transition must match its exact current
body AND project back to its original body through one named, narrow rewrite.
This is source identity evidence, not runtime parity or executable capacity.
"""
import hashlib,json
from pathlib import Path
def item(text,marker,derive=False):
 at=text.index(marker);start=text.rfind('#[derive',0,at) if derive else at;brace=text.index('{',at);depth=0
 for end in range(brace,len(text)):
  if text[end]=='{':depth+=1
  elif text[end]=='}':
   depth-=1
   if depth==0:return text[start:end+1]
 raise AssertionError(marker)
def canonical(text):
 return text.replace('crate::state_args::StateArgs','crate::args::StateArgs').replace('crate::run_profile::validate','bisect_runner::run_profile::validate').replace('use crate::args::{SearchMode as SeM, StructureMode as SM};\n        use crate::state_args::WeightMode as WM;','use crate::args::{SearchMode as SeM, StructureMode as SM, WeightMode as WM};')
BASELINE='docs/specs/wasm-native-state-source-baseline.json'
TRANSITIONS='docs/specs/wasm-native-state-source-transitions.json'
PROJECTION='native-replay-serde-and-count-domain-v1'

def original_projection(value):
 # These additions are the entire reviewed transition. Unknown attributes,
 # fields, variants, values and defaults remain visible to the original hash.
 value=value.replace(', serde::Serialize, serde::Deserialize','')
 value=value.replace(', serde::Serialize','')
 value=value.replace('#[serde(deny_unknown_fields)]\n','')
 value=value.replace('    #[serde(with = "crate::native_replay_recipe::optional_seed")]\n','')
 for kind in ['SeedCompositor','SplitStrategy','AlgorithmConfig']:
  value=value.replace(kind+'<C = usize>',kind).replace(kind+'<C>',kind)
 return value.replace(': C',': usize')

def check(root=Path('.')):
 baseline=root/BASELINE
 spec=json.loads(baseline.read_text(encoding='utf-8'))
 assert spec['schema_version']==1,'Unknown original extraction schema'
 reviewed=json.loads((root/TRANSITIONS).read_text(encoding='utf-8'))
 assert reviewed['schema_version']==1,'Unknown extraction transition schema'
 assert reviewed['baseline']==BASELINE
 assert reviewed['baseline_sha256']==hashlib.sha256(baseline.read_bytes()).hexdigest(),'Original extraction baseline changed'
 by_marker={row['marker']:row for row in reviewed['items']}
 assert len(by_marker)==len(reviewed['items']),'Duplicate transition marker'
 original={row['marker']:row for row in spec['items']}
 assert by_marker.keys()<=original.keys(),'Unknown transition marker'
 unchanged=0
 for entry in spec['items']:
  value=canonical(item((root/entry['current_source']).read_text(encoding='utf-8'),entry['marker'],entry['derive']))
  digest=hashlib.sha256(value.encode()).hexdigest()
  transition=by_marker.get(entry['marker'])
  if transition is None:
   assert digest==entry['sha256'],entry['marker']+' changed from the reviewed native body'
   unchanged+=1
   continue
  assert transition['original_sha256']==entry['sha256'],'Transition changed original body identity'
  assert transition['current_source']==entry['current_source']
  assert transition['projection']==PROJECTION,'Unknown reviewed projection'
  assert transition['rationale'].strip() and transition['review'].strip()
  assert (root/transition['review']).is_file(),'Missing source transition review'
  assert digest==transition['current_sha256'],entry['marker']+' changed from the reviewed current body'
  projected=hashlib.sha256(original_projection(value).encode()).hexdigest()
  assert projected==entry['sha256'],entry['marker']+' does not project to the original reviewed body'
 assert 'pub use bisect_runner::state_args::' in (root/'crates/bisect-cli/src/args.rs').read_text(encoding='utf-8')
 assert 'pub use bisect_runner::state_config::' in (root/'crates/bisect-cli/src/runner/mod.rs').read_text(encoding='utf-8')
 return dict(original_unchanged=unchanged,reviewed_transitions=len(by_marker),original_baseline_preserved=True)

if __name__=='__main__':
 import argparse
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--repo',type=Path,default=Path('.'))
 result=check(parser.parse_args().repo)
 print('PASS native State extraction: '+json.dumps(result)+'; source identity only, not runtime parity')
