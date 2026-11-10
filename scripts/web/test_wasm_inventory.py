"""Actual command scanner must preserve history while updating declaration identity."""
from pathlib import Path
import json,shutil,subprocess,sys,tempfile,unittest

ROOT=Path(__file__).resolve().parents[2]
DECLARATIONS=['crates/bisect-cli/src/args.rs','crates/bisect-cli/src/ceremony_cmd.rs','crates/bisect-cli/src/civic.rs','crates/bisect-cli/src/research.rs','crates/rplan-cli/src/main.rs','crates/rcount-cli/src/main.rs','crates/rhist-cli/src/main.rs','crates/bisect-tui/src/app.rs']
class InventoryHistory(unittest.TestCase):
 def fixture(self):
  parent=ROOT/'target/wasm-inventory-history-tests';parent.mkdir(parents=True,exist_ok=True)
  root=Path(tempfile.mkdtemp(prefix='case-',dir=parent));(root/'scripts/web').mkdir(parents=True);(root/'docs/specs').mkdir(parents=True)
  shutil.copy2(ROOT/'scripts/web/wasm_inventory.py',root/'scripts/web/wasm_inventory.py')
  for name in DECLARATIONS:
   dest=root/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/name,dest)
  data=json.loads((ROOT/'docs/specs/wasm-capability-inventory.json').read_text(encoding='utf-8'));file=root/'docs/specs/wasm-capability-inventory.json'
  return root,file,data
 def run_scanner(self,root):
  result=subprocess.run([sys.executable,str(root/'scripts/web/wasm_inventory.py')],cwd=root,capture_output=True)
  (root/'scanner.stdout').write_bytes(result.stdout);(root/'scanner.stderr').write_bytes(result.stderr)
  return result
 def test_root_and_entry_history_survive_real_scanner(self):
  root,file,data=self.fixture();data['recorded_scope_notes']=[{'id':'historical-test-note','scope':'Retain original evidence; no current execution claim.'}];data['future_history_extension']={'nested':[1,'unchanged']}
  first=data['entries'][0];first['future_entry_history']={'original':'retain this'};first['review']='original review';first['limitation']='original unresolved limitation';first['source']='obsolete-declaration.rs';first['enum']='ObsoleteEnum'
  file.write_text(json.dumps(data),encoding='utf-8');result=self.run_scanner(root);self.assertEqual(result.returncode,0,result.stderr)
  after=json.loads(file.read_text(encoding='utf-8'))
  for key in ['recorded_scope_notes','future_history_extension']:self.assertEqual(after[key],data[key])
  entry=next(e for e in after['entries'] if e['id']==first['id'])
  for key in ['future_entry_history','review','limitation','parity_record']:self.assertEqual(entry[key],first[key])
  self.assertNotEqual(entry['source'],'obsolete-declaration.rs');self.assertNotEqual(entry['enum'],'ObsoleteEnum')
 def test_unknown_schema_refuses_without_overwriting_history(self):
  root,file,data=self.fixture();data['schema_version']=999;raw=json.dumps(data).encode();file.write_bytes(raw);result=self.run_scanner(root)
  self.assertNotEqual(result.returncode,0);self.assertIn(b'Unsupported command inventory schema',result.stderr);self.assertEqual(file.read_bytes(),raw)
if __name__=='__main__':unittest.main()
