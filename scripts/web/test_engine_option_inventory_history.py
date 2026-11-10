"""Real engine declaration generator must retain historical extensions and refuse unknown schemas."""
from pathlib import Path
import json,shutil,subprocess,sys,tempfile,unittest

ROOT=Path(__file__).resolve().parents[2]
SOURCES=['crates/bisect-runner/src/run_args.rs','crates/bisect-runner/src/state_args.rs','crates/bisect-runner/src/states_args.rs','crates/bisect-runner/src/config.rs','crates/bisect-wasm/src/engine.rs']
class EngineInventoryHistory(unittest.TestCase):
 def fixture(self):
  parent=ROOT/'target/engine-option-history-tests';parent.mkdir(parents=True,exist_ok=True)
  root=Path(tempfile.mkdtemp(prefix='case-',dir=parent));(root/'scripts/web').mkdir(parents=True);(root/'docs/specs').mkdir(parents=True)
  shutil.copyfile(ROOT/'scripts/web/wasm_engine_option_inventory.py',root/'scripts/web/wasm_engine_option_inventory.py')
  for name in SOURCES:
   dest=root/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
  data=json.loads((ROOT/'docs/specs/wasm-engine-option-inventory.json').read_text(encoding='utf-8'))
  return root,root/'docs/specs/wasm-engine-option-inventory.json',data
 def run_generator(self,root):
  result=subprocess.run([sys.executable,str(root/'scripts/web/wasm_engine_option_inventory.py')],cwd=root,capture_output=True)
  (root/'generator.stdout').write_bytes(result.stdout);(root/'generator.stderr').write_bytes(result.stderr)
  return result
 def test_root_and_entry_extensions_survive_changed_declaration(self):
  root,file,data=self.fixture();data['recorded_scope_notes']=[{'scope':'Original receipt remains original.'}];data['future_root_history']={'nested':[1,'retain']}
  row=data['entries'][0];row['future_entry_history']={'original':'retain'};row['review']='Original review';row['rust_type']='ObsoleteRecordedType'
  file.write_bytes(json.dumps(data).encode());result=self.run_generator(root);self.assertEqual(result.returncode,0,result.stderr)
  after=json.loads(file.read_text(encoding='utf-8'))
  for key in ('recorded_scope_notes','future_root_history'):self.assertEqual(after[key],data[key])
  actual=next(r for r in after['entries'] if r['id']==row['id'])
  for key in ('future_entry_history','review','evidence','limitation','parity_record'):self.assertEqual(actual[key],row[key])
  self.assertNotEqual(actual['rust_type'],'ObsoleteRecordedType')
 def test_unknown_schema_refuses_without_overwriting_original(self):
  root,file,data=self.fixture();data['schema_version']=999;original=json.dumps(data).encode();file.write_bytes(original)
  result=self.run_generator(root);self.assertNotEqual(result.returncode,0)
  self.assertIn(b'Unsupported engine option inventory schema',result.stderr);self.assertEqual(file.read_bytes(),original)
if __name__=='__main__':unittest.main()
