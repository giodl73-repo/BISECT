"""Freeze the actual Run parser and consumed native configuration boundary."""
from pathlib import Path
import re
import json
import hashlib

shared = Path('crates/bisect-runner/src/run_args.rs').read_text(encoding='utf-8')
legacy = Path('crates/bisect-cli/examples/support/run_args_legacy.rs').read_text(encoding='utf-8')
for name, kind in [('RunArgs', 'struct'), ('Stage', 'enum'), ('RunType', 'enum')]:
    pattern = r'pub '+kind+' '+name+r' \{.*?\n\}'
    assert re.search(pattern, shared, re.S).group() == re.search(pattern, legacy, re.S).group(), name
main = Path('crates/bisect-cli/src/main.rs').read_text(encoding='utf-8')
branch = main.split('Commands::Run(args) => {',1)[1].split('Commands::Analyze(args) => {',1)[0]
assert 'cfg.apply_run_args(&args)' in branch
configuration = Path('crates/bisect-runner/src/run_command.rs').read_text(encoding='utf-8').split('pub fn years(',1)[0]
reads = sorted(set(re.findall(r'args\.([a-z_]+)',branch+configuration)))
consumed = ['debug','output_dir','partition_mode','reprocess','reset','states','version','workers','year']
assert reads == consumed, reads
unused = ['dpi','election_year','skip_analysis','skip_political','skip_demographic','stages','print_only','processing_mode','minimum_boundary_length','run_type','experiment_name']
for field in unused:
    assert field not in reads
root = Path('target/native-run-command');root.mkdir(parents=True,exist_ok=True)
(root/'source-guard.json').write_text(json.dumps({'declarations_unchanged':True,'consumed_fields':consumed,'parsed_but_unused_fields':unused,'native_run_branch_sha256':hashlib.sha256(branch.encode()).hexdigest(),'shared_configuration_sha256':hashlib.sha256(configuration.encode()).hexdigest(),'scope':'Source preservation and native consumed-field baseline; execution parity requires the independent native/wasm cases.'},indent=2)+'\n',encoding='utf-8')
print('PASS actual native Run extraction: exact frozen declarations/enums, shared native config delegation and nine consumed/eleven unused fields.')
