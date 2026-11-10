"""Mutation controls for explicit source transitions; owned fixtures are retained."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
module_spec=importlib.util.spec_from_file_location('state_extraction',Path(__file__).with_name('test_native_state_extraction.py'))
extraction=importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(extraction)

class SourceTransitions(unittest.TestCase):
 def fixture(self):
  parent=ROOT/'target/native-state-extraction-transition-tests'
  parent.mkdir(parents=True,exist_ok=True)
  root=Path(tempfile.mkdtemp(prefix='case-',dir=parent))
  baseline=json.loads((ROOT/extraction.BASELINE).read_text(encoding='utf-8'))
  transitions=json.loads((ROOT/extraction.TRANSITIONS).read_text(encoding='utf-8'))
  names={extraction.BASELINE,extraction.TRANSITIONS,'crates/bisect-cli/src/args.rs','crates/bisect-cli/src/runner/mod.rs'}
  names.update(row['current_source'] for row in baseline['items'])
  names.update(row['review'] for row in transitions['items'])
  for name in names:
   target=root/name;target.parent.mkdir(parents=True,exist_ok=True)
   shutil.copy2(ROOT/name,target)
  return root

 def edit_transition(self,root,change):
  file=root/extraction.TRANSITIONS
  data=json.loads(file.read_text(encoding='utf-8'));change(data)
  file.write_text(json.dumps(data,indent=2),encoding='utf-8')

 def change_seed(self,root,before,after,repin=False):
  file=root/'crates/bisect-runner/src/state_config.rs'
  text=file.read_text(encoding='utf-8');assert before in text
  file.write_text(text.replace(before,after,1),encoding='utf-8')
  if repin:
   body=extraction.canonical(extraction.item(file.read_text(encoding='utf-8'),'pub enum SeedCompositor',True))
   digest=hashlib.sha256(body.encode()).hexdigest()
   self.edit_transition(root,lambda data:next(row for row in data['items'] if row['marker']=='pub enum SeedCompositor').update(current_sha256=digest))

 def test_current_bodies_and_original_hashes(self):
  self.assertEqual(extraction.check(self.fixture()),dict(original_unchanged=16,reviewed_transitions=6,original_baseline_preserved=True))

 def test_unreviewed_current_body_refuses(self):
  root=self.fixture();self.change_seed(root,'Multi { seeds: C }','Multi { seeds: u64 }')
  with self.assertRaisesRegex(AssertionError,'reviewed current body'):extraction.check(root)

 def test_repinned_type_change_still_refuses_original_projection(self):
  root=self.fixture();self.change_seed(root,'Multi { seeds: C }','Multi { seeds: u64 }',True)
  with self.assertRaisesRegex(AssertionError,'does not project'):extraction.check(root)

 def test_repinned_variant_change_still_refuses(self):
  root=self.fixture();self.change_seed(root,'    Single,','    ChangedSingle,',True)
  with self.assertRaisesRegex(AssertionError,'does not project'):extraction.check(root)

 def test_unknown_serde_attribute_is_not_erased(self):
  root=self.fixture();self.change_seed(root,'#[serde(deny_unknown_fields)]','#[serde(rename_all = "snake_case")]',True)
  with self.assertRaisesRegex(AssertionError,'does not project'):extraction.check(root)

 def test_unchanged_body_stays_exact(self):
  root=self.fixture();file=root/'crates/bisect-runner/src/state_config.rs'
  text=file.read_text(encoding='utf-8');marker='impl Default for AlgorithmConfig';at=text.index('{',text.index(marker))
  file.write_text(text[:at+1]+'\n// unexpected source change\n'+text[at+1:],encoding='utf-8')
  with self.assertRaisesRegex(AssertionError,'reviewed native body'):extraction.check(root)

 def test_original_baseline_change_refuses(self):
  root=self.fixture();file=root/extraction.BASELINE;file.write_bytes(file.read_bytes()+b'\n')
  with self.assertRaisesRegex(AssertionError,'Original extraction baseline changed'):extraction.check(root)

 def test_unknown_transition_schema_refuses(self):
  root=self.fixture();self.edit_transition(root,lambda data:data.update(schema_version=999))
  with self.assertRaisesRegex(AssertionError,'Unknown extraction transition schema'):extraction.check(root)

 def test_transition_cannot_change_original_hash(self):
  root=self.fixture();self.edit_transition(root,lambda data:data['items'][0].update(original_sha256='0'*64))
  with self.assertRaisesRegex(AssertionError,'original body identity'):extraction.check(root)

 def test_duplicate_transition_refuses(self):
  root=self.fixture();self.edit_transition(root,lambda data:data['items'].append(copy.deepcopy(data['items'][0])))
  with self.assertRaisesRegex(AssertionError,'Duplicate transition marker'):extraction.check(root)

 def test_unknown_projection_refuses(self):
  root=self.fixture();self.edit_transition(root,lambda data:data['items'][0].update(projection='erase-arbitrary-changes'))
  with self.assertRaisesRegex(AssertionError,'Unknown reviewed projection'):extraction.check(root)

 def test_missing_transition_refuses(self):
  root=self.fixture();self.edit_transition(root,lambda data:data['items'].pop(0))
  with self.assertRaisesRegex(AssertionError,'reviewed native body'):extraction.check(root)

 def test_missing_review_refuses(self):
  root=self.fixture();self.edit_transition(root,lambda data:data['items'][0].update(review='absent.md'))
  with self.assertRaisesRegex(AssertionError,'Missing source transition review'):extraction.check(root)

if __name__=='__main__':unittest.main()
