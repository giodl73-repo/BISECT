"""Freeze reviewed label installation evidence and final-package bytes, never replace snapshots."""
import argparse,hashlib,json,shutil,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def local(name):
 p=Path(name);p=(p if p.is_absolute() else ROOT/p).resolve();assert p.is_relative_to(ROOT);return p
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--package',required=True);parser.add_argument('--default-browser',required=True);parser.add_argument('--snapshot-name');parser.add_argument('--max-storage-bytes',type=int);parser.add_argument('--dry-run',action='store_true');args=parser.parse_args()
 package=local(args.package);assert package.is_relative_to(ROOT/'dist')
 snapshot_name=args.snapshot_name or package.name
 assert re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,149}',snapshot_name) and not snapshot_name.endswith('.')
 destination=ROOT/'target/label-repository-delivery/package-snapshots'/snapshot_name
 assert not destination.exists(),'Preserve prior snapshots; choose a new package'
 files=set()
 def include(name,expected=None):
  p=local(name);assert p.is_file(),str(p)
  if expected:assert sha(p)==expected,'Changed evidence: '+str(p)
  files.add(p)
 def ref(value):include(value['path'],value['sha256'])
 def tree(folder):
  for p in local(folder).rglob('*'):
   if p.is_file():include(p)
 record=ROOT/'target/native-workspace-source/package-evidence.json';bundle=json.loads(record.read_text(encoding='utf-8'))
 assert local(bundle['package'])==package and bundle['browser_status']=='passed' and bundle['deployed'] is False
 include(record)
 for value in bundle['source_files']+bundle['results']+bundle.get('explicit_c_batch_evidence',[]):ref(value)
 if bundle.get('optional_c_browser_evidence'):ref(bundle['optional_c_browser_evidence'])
 ref(bundle['offline_manifest']);offline=json.loads((package/'offline-manifest.json').read_text(encoding='utf-8'));assert offline['version']==bundle['offline_version'];tree(package)
 engine=sha(package/'toolkit/bisect_wasm.wasm');profiles={}
 label=bundle['label_repository_evidence'];ref(label['node']);ref(label['browser'])
 node=json.loads(local(label['node']['path']).read_text(encoding='utf-8'));browser=json.loads(local(label['browser']['path']).read_text(encoding='utf-8'))
 assert node['wasm']['sha256']==engine and browser['wasm']['sha256']==engine
 assert local(browser['package'])==package/'toolkit' and browser['source_tree'] is False
 assert all(browser[k] for k in ['actual_worker_and_opfs','inert_open','review_no_writes','explicit_confirmation','edits_retire_review','cancel_after_real_write','explicit_continuation','analysis_report_exact_bytes','registry_last','native_cli_consumption'])

 for value in browser['served_assets']:ref(value)
 include(local(label['node_root'])/'execute_tool_request.exe',node['native_host_sha256']);include(local(label['browser_root'])/'bisect.exe',browser['native_sha256']);tree(label['node_root']);tree(label['browser_root'])
 for name in ['default','c-sidecar']:
  folder=ROOT/'target/cffi-legacy-gates'/name;gate=json.loads((folder/'build.json').read_text(encoding='utf-8-sig'));assert gate['status']=='passed';profiles[name]=gate['wasm_sha256']
  for value in gate['source_files']:ref(value)

  if gate.get('post_build_label_revalidation'):
   post=gate['post_build_label_revalidation']
   for key in ['original_build_record','node_evidence','browser_evidence']:ref(post[key])
   tree(local(post['node_evidence']['path']).parent);tree(local(post['browser_evidence']['path']).parent)
  include(folder.parent/(name+'.wasm'),gate['wasm_sha256']);include(folder.parent/'state-core-reference.exe',gate['native_oracle_sha256']);tree(folder)
  run=local(gate['label_repository_evidence']['root']);evidence=json.loads((run/'evidence.json').read_text(encoding='utf-8'));assert evidence['wasm']['sha256']==gate['wasm_sha256'];assert evidence['native_cli_consumption'] and evidence['native_cli_tampered_refusal'];include(run/'bisect.exe',evidence['native_sha256']);include(run/'execute_tool_request.exe',evidence['native_host_sha256']);tree(run)
 assert engine==profiles['c-sidecar'] and profiles['default']!=engine
 default=local(args.default_browser);evidence=json.loads((default/'evidence.json').read_text(encoding='utf-8'));assert evidence['wasm']['sha256']==profiles['default'] and evidence['actual_worker_and_opfs'];tree(default)
 for value in evidence['served_assets']:ref(value)
 include(default/'bisect.exe',evidence['native_sha256'])
 for name in ['README.md','docs/specs/wasm-desktop-parity-plan.md','docs/specs/wasm-parity-ledger.md','docs/specs/wasm-label-repository-installation.md','docs/specs/wasm-practitioner-actions.json','docs/specs/wasm-engine-option-inventory.json','scripts/web/snapshot_label_repository_installation.py','target/label-repository-delivery/dual-build.log','target/label-repository-delivery/package-build.log','target/label-repository-delivery/project-regression.log','design/pitfalls/pitfalls-pipeline.md','design/pitfalls/README.md','target/label-repository-delivery/package-snapshots/wasm-label-repository-validated/failed-attempt.json','target/label-repository-delivery/capacity-refusal.log']:include(name)
 records=[];blobs={}
 for p in sorted(files):
  identity=sha(p);size=p.stat().st_size
  if identity in blobs:assert blobs[identity].stat().st_size==size
  else:blobs[identity]=p
  records.append(dict(path=p.relative_to(ROOT).as_posix(),sha256=identity,byte_length=size,blob='blobs/'+identity))
 required=sum(p.stat().st_size for p in blobs.values());free=shutil.disk_usage(ROOT).free
 if args.max_storage_bytes is not None:
  assert args.max_storage_bytes>0;free=min(free,args.max_storage_bytes)
 reserve=16*1024*1024
 capacity=dict(files=len(records),unique_blobs=len(blobs),source_bytes=sum(row['byte_length'] for row in records),stored_bytes=required,free_bytes=free,reserve_bytes=reserve)
 if required+reserve>free:raise SystemExit('Snapshot capacity refusal before creating output: '+json.dumps(capacity))
 if args.dry_run:print(json.dumps(capacity));return
 destination.mkdir(parents=True);(destination/'blobs').mkdir()
 try:
  for identity,p in blobs.items():
   out=destination/'blobs'/identity;partial=out.with_suffix('.partial');shutil.copy2(p,partial);assert sha(partial)==identity;partial.rename(out)
  for row in records:assert sha(ROOT/row['path'])==row['sha256'],'Source changed while freezing: '+row['path']
  manifest=destination/'snapshot.json';manifest.write_text(json.dumps(dict(schema_version='bisect-label-repository-snapshot-v1',storage_format='content-addressed-blobs-v1',storage=capacity,package=package.relative_to(ROOT).as_posix(),offline_version=offline['version'],wasm_identities=profiles,files=records,deployed=False,registration_boundary='Final roles review and capability-ledger snapshot references are registered after freezing these execution bytes; excluded to avoid circular hashes. No acceptance-v1 promotion.',scope='Scoped analysis/report repository application, actual native CLI consumption and exact merge references, both engine builds and final C-enabled browser package. Not whole native filesystem orchestration, OS-picker or release/full-parity acceptance.'),indent=2)+'\n',encoding='utf-8')
 except Exception as error:
  (destination/'failed-attempt.json').write_text(json.dumps(dict(status='failed',error=str(error),storage=capacity),indent=2)+'\n',encoding='utf-8');raise
 print(json.dumps(dict(snapshot=str(manifest),**capacity)))
if __name__=='__main__':main()
