"""Check declared fixture order and direct native-executable build dependencies.

This is an early build-graph check, not runtime parity evidence. It checks
literal node/python invocations and native executable references; dynamic
commands, imports and conditional execution still need the full build gates.
"""
import argparse
import json
from pathlib import Path
import re
import tomllib

FIXTURE_PRODUCERS = {
    'test_external_label_analysis_wasm.mjs': 'test_analysis_package_wasm.mjs',
    'test_civic_inventory_wasm.mjs': 'test_civic_ingest_wasm.mjs',
    'test_engine_replay_wasm.mjs': 'test_multiscale_wasm.mjs',
    'test_replay_multiscale_bindings_wasm.mjs': 'test_multiscale_wasm.mjs',
    'test_tui_run_wasm.mjs': 'test_config_build_wasm.mjs',
    'test_prepared_engine.mjs': 'test_config_build_wasm.mjs',
    'test_native_character_engine_build.mjs': 'test_config_build_wasm.mjs',
    'test_batch_accepted_replay.mjs': 'test_config_build_wasm.mjs',
    'test_election_analysis_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_full_iowa_geometry_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_full_state_batch_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_full_state_compactness_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_interchange_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_pareto_context_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_pareto_selection_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_plan_import_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_plan_map_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_plan_splits_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_plan_statistics_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_reproducibility_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_suite_nesting_wasm.mjs': 'test_smc_real_state_wasm.mjs',
    'test_suite_stability_wasm.mjs': 'test_smc_real_state_wasm.mjs',
}


# Deliberately bounded to literal root+filename writes. This catches the actual
# cold-build omission before expensive runtime gates; it is not a JS interpreter.
FIXTURE_EMISSIONS = {
    'test_config_build_wasm.mjs': ('target/config-build-wasm', 'browser-retry.bisect'),
}


STATE_SMC_MARKERS = (
    "const root='target/smc-real-state-wasm'",
    "of [['RI',2,32],['IA',4,16],['NC',14,8]]",
    "fs.writeFile(root+'/'+state+'.rctx',",
    "fs.writeFile(root+'/'+state+'-ensemble.ndjson',",
    "fs.writeFile(root+'/'+state+'-selected.rplan',",
    "fs.writeFile(root+'/'+state+'-ensemble.bisect',",
)

def package_bins(root, package):
    crate = root / 'crates' / package
    manifest = crate / 'Cargo.toml'
    if not manifest.is_file():
        return []
    data = tomllib.loads(manifest.read_text(encoding='utf-8-sig'))
    names = [entry['name'] for entry in data.get('bin', [])]
    if not names and (crate / 'src/main.rs').is_file():
        names.append(data['package']['name'])
    names.extend(file.stem for file in (crate / 'src/bin').glob('*.rs'))
    return names


def check(root, builder='scripts/web/build_wasm.ps1'):
    root = Path(root)
    producers, gates, errors = {}, [], []
    for line_number, line in enumerate((root / builder).read_text(
            encoding='utf-8-sig').splitlines(), 1):
        if re.match(r'\s*cargo\s+(build|run)\b', line):
            profile = 'release' if re.search(r'(?<!\S)--release(?!\S)', line) else 'debug'
            examples = re.findall(r'--example\s+([\w-]+)', line)
            for name in examples:
                producers.setdefault(f'target/{profile}/examples/{name}.exe', []).append(line_number)
            bins = re.findall(r'--bin\s+([\w-]+)', line)
            package = re.search(r'-p\s+([\w-]+)', line)
            if not bins and not examples and package:
                bins = package_bins(root, package[1])
            for name in bins:
                producers.setdefault(f'target/{profile}/{name}.exe', []).append(line_number)
        command = re.match(r'\s*(?:node|python)\s+(scripts/web/[\w.-]+\.(?:mjs|py))', line)
        if command:
            script = root / command[1]
            if not script.is_file():
                errors.append({'script': command[1], 'line': line_number,
                               'error': 'invoked script is missing'})
                continue
            # Preserve non-UTF-8 bytes while recognizing ASCII path literals.
            text = script.read_text(encoding='utf-8-sig', errors='surrogateescape')
            refs = sorted(set(re.findall(
                r'target/(?:debug|release)/(?:examples/)?[\w.-]+\.exe', text)))
            gates.append({'script': command[1], 'line': line_number, 'references': refs})
    references = 0
    for gate in gates:
        for executable in gate['references']:
            references += 1
            earlier = [line for line in producers.get(executable, []) if line < gate['line']]
            if not earlier:
                errors.append({'script': gate['script'], 'line': gate['line'],
                               'executable': executable,
                               'error': 'native executable has no earlier build in its profile'})
    first = {}
    for gate in gates:
        first.setdefault(Path(gate['script']).name, gate['line'])
    fixture_edges = 0
    for consumer, producer in FIXTURE_PRODUCERS.items():
        if consumer not in first:
            continue
        fixture_edges += 1
        if producer not in first or first[producer] >= first[consumer]:
            errors.append({'script': consumer, 'producer': producer,
                           'error': 'fixture producer must run before consumer'})
    fixture_emissions = 0
    for producer, (directory, filename) in FIXTURE_EMISSIONS.items():
        if producer not in first:
            continue
        fixture_emissions += 1
        source = (root / 'scripts/web' / producer).read_text(encoding='utf-8-sig')
        root_literal = r'''const\s+root\s*=\s*['"]''' + re.escape(directory) + r'''['"]'''
        write_literal = r'''fs\.writeFile\(\s*root\s*\+\s*['"]/''' + re.escape(filename) + r'''['"]'''
        if not re.search(root_literal, source) or not re.search(write_literal, source):
            errors.append({'script': producer, 'artifact': directory + '/' + filename,
                           'error': 'declared shared fixture has no literal writer in producer'})
    smc_producer = 'test_smc_real_state_wasm.mjs'
    if smc_producer in first:
        source = (root / 'scripts/web' / smc_producer).read_text(encoding='utf-8-sig')
        fixture_emissions += len(STATE_SMC_MARKERS) - 2
        if any(marker not in source for marker in STATE_SMC_MARKERS):
            errors.append({'script': smc_producer, 'error': 'declared RI/IA/NC SMC fixture loop or file writer is missing'})
    return {'status': 'failed' if errors else 'passed', 'builder': builder,
            'script_invocations': len(gates), 'native_references_checked': references,
            'fixture_edges_checked': fixture_edges, 'fixture_emissions_checked': fixture_emissions, 'errors': errors,
            'scope': 'Static direct-command dependency checks only; no runtime or parity acceptance.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--builder', default='scripts/web/build_wasm.ps1')
    args = parser.parse_args()
    result = check(args.root, args.builder)
    print(json.dumps(result))
    return 0 if result['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
