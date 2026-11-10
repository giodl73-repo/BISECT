"""Audit current native States consumption before claiming missing browser parity."""
import hashlib
import json
import re
from pathlib import Path

main_path = Path('crates/bisect-cli/src/main.rs')
config_path = Path('crates/bisect-runner/src/states_config.rs')
main = main_path.read_text(encoding='utf-8')
branch = main.split('Commands::States(args) => {', 1)[1].split('Commands::Run(args) => {', 1)[0]
config = config_path.read_text(encoding='utf-8').split('pub fn from_args(args:&StatesArgs)->Self {', 1)[1].split('\npub fn ', 1)[0]
unused = ['run_analysis', 'skip_political', 'skip_demographic', 'election_year', 'processing_mode']
reads = sorted(set(re.findall(r'args\.([a-z_]+)', branch + config)))
assert 'workers' in reads and 'run_states_parallel(to_run, args.workers)' in branch
assert 'cfg.apply_states_args(&args)' in branch
for field in unused:
    assert field not in reads, f'Native States now consumes {field}; update the browser parity review.'
    # These would signal that another native runner layer acquired the option.
    for source in Path('crates/bisect-cli/src/runner').rglob('*.rs'):
        assert not re.search(r'\b' + field + r'\b', source.read_text(encoding='utf-8'))
report = {'schema_version':'bisect-native-states-option-consumption-audit-v1',
          'native_entry_source':str(main_path), 'shared_configuration_source':str(config_path),
          'entry_branch_sha256':hashlib.sha256(branch.encode()).hexdigest(),
          'shared_configuration_sha256':hashlib.sha256(config.encode()).hexdigest(),
          'consumed_fields':reads, 'parsed_but_unconsumed_followup_fields':unused,
          'requested_workers_forwarded':True,
          'scope':'Current native States entry branch, its shared converter and runner field-use guard. This is a source-consumption finding, not a claim that undocumented follow-up behavior exists. Browser workflow controls provide separate explicit analysis/map/report actions.'}
root = Path('target/native-states-workers'); root.mkdir(parents=True, exist_ok=True)
(root / 'native-option-consumption.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print('PASS native States consumption audit: worker count forwarded; five parsed follow-up/processing flags remain unused in native execution.')
