"""Check the retained native, Workbench and folder results for scoped State actions."""
import base64
import hashlib
import itertools
import json
from pathlib import Path
import sys


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def files_for(row, key):
    return {p: base64.b64decode(v, validate=True) for p, v in row[key].items()}


def standalone(reg):
    suite = reg / 'target/native-state-data-reset'
    native, browser = (read(suite / 'evidence.json'),
                       read(reg / 'target/native-state-data-reset-browser/evidence.json'))
    assert native['native_cli_filesystem_comparisons'] == 32 and native['refusals'] == 25
    assert native['reset_partitions'] == browser['partition_runs'] == 0
    assert native['wasm'] == browser['wasm']
    assert all(browser[k] for k in ('real_opfs_with_injected_picker','stale_target_refused_without_removal',
               'only_data_subtree_removed','root_files_and_manifest_preserved','native_labeled_force_and_incomplete_order',
               'unabortable_removal_reported_interrupted','force_and_review_retired_on_open','accepted_download_remains_available'))
    rows = read(suite / 'cli-records.json')
    actual = {tuple(row[k] for k in ('layout','manifest','incomplete','force','reset')) for row in rows}
    assert len(rows) == 32 and actual == set(itertools.product(('state','labeled'), *[(False,True)]*4))
    totals = {'incomplete':0,'collision':0,'missing-adjacency':0}
    for row in rows:
        layout, manifest, incomplete, force, reset = (row[k] for k in ('layout','manifest','incomplete','force','reset'))
        reason = ('incomplete' if incomplete else
                  'collision' if layout == 'labeled' and manifest and not force else 'missing-adjacency')
        expected_message = {'incomplete':'Incomplete plan','collision':'already exists','missing-adjacency':'adjacency load failed'}[reason]
        assert row['native_exit_code'] == 1 and expected_message in row['native_stderr']
        assert row['expected_refusal'] == (None if reason == 'missing-adjacency' else reason)
        before = files_for(row, 'initial_files')
        after = {p:v for p,v in before.items() if not (reset and reason == 'missing-adjacency' and p.startswith('data/'))}
        assert files_for(row, 'native_after_files') == files_for(row, 'browser_after_files') == after
        totals[reason] += 1
    return {'native_failures':totals,'native_sha256':native['native_executable']['sha256'],'wasm':native['wasm']}


def before_run(reg):
    suite = reg / 'target/native-state-target-run'
    browser_dir = reg / 'target/native-state-target-run-browser'
    native, browser = read(suite/'evidence.json'),read(browser_dir/'evidence.json')
    assert native['wasm'] == browser['wasm'] and native['actual_successful_cli_run']
    assert all(native[k] for k in ('required_backend_refused_before_action','grant_replay_refused',
               'scope_count_refused_before_action','saved_derived_binding_without_authority_refused','incomplete_target_blocks_run',
               'force_alone_preserves_data','post_reset_source_failure_retains_action_receipt'))
    assert all(browser[k] for k in ('actual_worker_and_opfs','inert_open_with_flags_and_action_receipt',
               'stale_nonce_ignored_before_progress','changed_target_refuses_before_run','reset_only_data_before_computation',
               'cancel_pending_run_does_not_remove_data','cancel_after_removal_retains_interrupted_receipt'))
    records = read(suite / 'records.json')
    assert len(records)==1 and records[0]['native_exit_code']==0
    original = records[0]
    saved = read(browser_dir / 'completed.bisect')
    run = saved['result']
    assert run['prepared']['result']['assignments'] == original['wasm_run']['prepared']['result']['assignments']
    units = run['prepared']['export']['document']['plan']['units']['unit_ids']
    assert {str(i):run['prepared']['result']['assignments'][u] for i,u in enumerate(units)} == original['native_assignments']
    receipt = read(browser_dir / 'action-receipt.json')
    assert receipt['intent']['source_arguments_json'] == saved['files']['stateCommandInputs']['arguments_json']
    assert json.loads(receipt['intent']['source_arguments_json']) == original['original_arguments']
    assert receipt['receipt']['reset_receipt']['status'] == 'completed'
    assert read(browser_dir/'interrupted-action-receipt.json')['receipt']['status']=='interrupted'
    rows=read(suite/'backend-cases.json')
    assert len(rows)==10 and all(r['executed'] and r['native_exit_code']==0 for r in rows)
    for row in rows:
        assert {str(i):row['assignments'][u] for i,u in enumerate(units)}==row['native_assignments']
    assert read(suite/'failed-after-reset.json')['native_target_actions']['receipt']['status']=='completed'
    return {'native_assignment_matches':11,'native_sha256':native['native_binary']['sha256'],'wasm':native['wasm']}


def custom(root, profile):
    raw = root / (profile+'-readback')
    browser = root / (profile+'-browser-r2')
    native, observed = read(raw/'evidence.json'),read(browser/'evidence.json')
    assert native['wasm']==observed['wasm'] and observed['open_inert'] and not observed['actual_os_picker_permission']
    assert native['unsafe_name_refused_before_authority']
    assert [c['case'] for c in native['cases']] == [c['case'] for c in observed['cases']]
    for case in native['cases']:
        name=case['case']
        native_target=raw/(name+'-native')/case['target']
        plan=read(native_target/'plan.rplan')
        prepared=read(browser/(name+'.bisect'))['result']['prepared']
        assert prepared['export']['document']['metadata']['label']==plan['metadata']['label']==case['expectedLabel']
        units=plan['plan']['units']['unit_ids']
        assert prepared['result']['assignments']==case['assignments']==dict(zip(units,(i+1 for i in plan['plan']['assignment']),strict=True))
        assert (native_target/'keep.txt').read_bytes()==b'keep'
        assert (native_target/'data/old.txt').exists()!=case['reset']
    return {'cases':len(native['cases']),'native_sha256':native['native_sha256'],'wasm':native['wasm']}


def main(root, output):
    result={}
    for p in ('core','c'):
        reg=root/(p+'-regressions')
        result[p]={'standalone':standalone(reg),'before_run':before_run(reg),'custom':custom(root,p)}
        assert len({r['native_sha256'] for r in result[p].values()})==1
        assert len({r['wasm']['sha256'] for r in result[p].values()})==1
    with output.open('x',encoding='utf8') as f:
        json.dump({'status':'passed-independent-state-target-action-readback','profiles':result,
                   'os_picker_permission_tested':False,'full_parity':False},f,indent=2)


if __name__=='__main__':
    main(*map(Path,sys.argv[1:]))
