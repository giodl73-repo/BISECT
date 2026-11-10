"""Observe real CLI population-source effects without assuming vector selection."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),
                sha256=hashlib.sha256(data).hexdigest(), byte_length=len(data))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', default='target/release/bisect.exe')
    parser.add_argument('--engine', choices=['metis-core', 'c-ffi'], default='metis-core')
    args = parser.parse_args()
    native = (ROOT / args.native).resolve()
    assert native.is_relative_to(ROOT) and native.is_file()
    output = ROOT / 'target/population-source' / ('native-' + str(time.time_ns()))
    output.mkdir(parents=True)
    cli = output / 'bisect-reference.exe'
    original = pin(native)
    shutil.copy2(native, cli)
    assert pin(native) == original and pin(cli)['sha256'] == original['sha256']
    fixture_path = ROOT / 'scripts/web/fixtures/native-state-source.json'
    fixture = json.loads(fixture_path.read_text(encoding='utf-8'))['nativeAdjacencyInputs']
    sources = output / 'sources'
    sources.mkdir()
    for key, suffix in [('adjacency', '.adj.bin'), ('geoids', '_geoids.json'), ('centroids', '_centroids.json')]:
        (sources / ('fixture' + suffix)).write_bytes(base64.b64decode(fixture[key]['base64'], validate=True))
    cases = [('total', 'total', 'absent', None), ('vap', 'vap', 'absent', None),
             ('prison-adjusted', 'prison_adjusted', 'absent', None),
             ('unrecognized', 'unrecognized_native_value', 'absent', None),
             ('cvap-missing', 'cvap', 'absent', None),
             ('cvap-empty-file', 'cvap', 'empty-file', None),
             ('cvap-directory', 'cvap', 'directory', None),
             ('cvap-valid-file', 'cvap', 'valid-file', None),
             ('total-with-cvap-file', 'total', 'valid-file', None),
             ('cvap-uppercase', 'CVAP', 'absent', None),
             ('cvap-raw-name', 'cvap', 'empty-file', 'Research Area')]
    baseline, outcomes = None, []
    for name, requested, presence, raw_name in cases:
        work = output / name
        work.mkdir()
        effective_name = raw_name or 'rhode_island'
        relative = Path('data/2020/demographics') / (effective_name + '_cvap_2020.csv')
        cvap = work / relative
        if presence != 'absent':
            cvap.parent.mkdir(parents=True)
            if presence == 'directory':
                cvap.mkdir()
            elif presence == 'valid-file':
                ids = json.loads(base64.b64decode(fixture['geoids']['base64']))
                cvap.write_text('GEOID,total_vap,minority_vap\n' + ''.join(
                    f'{ids[str(i)]},{20+i},{5+i}\n' for i in range(len(ids))), encoding='utf-8')
            else:
                cvap.write_bytes(b'')
        arguments = ['state', '--state', 'RI', '--year', '2020', '--adjacency',
                     str(sources / 'fixture.adj.bin'), '--districts', '2',
                     '--metis-engine', args.engine, '--structure', 'nway',
                     '--search', 'single', '--seed', '18446744073709551615',
                     '--balance-tolerance', '5', '--population-source', requested,
                     '--output-dir', str(work / 'outputs'), '--manifest']
        if raw_name is not None:
            arguments.extend(['--state-name', raw_name])
        completed = subprocess.run([str(cli), *arguments], cwd=work,
                                   env={k: v for k, v in os.environ.items() if k != 'BISECT_MANIFEST'},
                                   capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=120)
        (work / 'stdout.log').write_text(completed.stdout, encoding='utf-8')
        (work / 'stderr.log').write_text(completed.stderr, encoding='utf-8')
        expected_failure = presence in ['empty-file', 'directory'] and raw_name is None
        assert (completed.returncode != 0) == expected_failure, name
        target = work / 'outputs/2020/states' / effective_name
        warning = 'CVAP data not found' in completed.stderr
        assert warning == (requested == 'cvap' and presence == 'absent'), name
        if expected_failure:
            assert 'VAP demographics load failed' in completed.stderr, name
            assert not (target / 'manifest.json').exists(), name
            outcomes.append(dict(name=name, arguments=arguments, exit_code=completed.returncode,
                                 requested=requested, presence=presence,
                                 expected_relative_cvap_path=relative.as_posix(),
                                 missing_cvap_warning=warning, expected_native_failure=True,
                                 failure_stage='audit-sidecar-demographics',
                                 files=[pin(p) for p in sorted(work.rglob('*')) if p.is_file()]))
            continue
        plan = json.loads((target / 'plan.rplan').read_text(encoding='utf-8'))
        context = json.loads((target / 'context.rctx').read_text(encoding='utf-8'))
        manifest = json.loads((target / 'manifest.json').read_text(encoding='utf-8'))
        semantic = dict(assignment=plan['plan']['assignment'], units=plan['plan']['units'],
                        populations=context['populations'])
        if baseline is None:
            baseline = semantic
        assert semantic == baseline, name + ' unexpectedly changed the assigned graph'
        assert manifest['population_source'] == requested, name + ' did not retain requested metadata'
        assert bool(context.get('demographics')) == (presence == 'valid-file'), name
        outcomes.append(dict(name=name, arguments=arguments, exit_code=completed.returncode,
                             requested=requested, presence=presence, expected_relative_cvap_path=relative.as_posix(),
                             missing_cvap_warning=warning, expected_native_failure=False,
                             assignments_units_and_populations_equal_baseline=True,
                             report_demographics_loaded=bool(context.get('demographics')),
                             manifest_records_requested_value=True,
                             files=[pin(p) for p in sorted(work.rglob('*')) if p.is_file()]))
    record = dict(status='passed', native=pin(cli), copied_from=original, requested_metis_engine=args.engine,
                  fixture=pin(fixture_path), producer=pin(Path(__file__).resolve()),
                  reviewed_sources=[pin(ROOT / name) for name in [
                      'crates/bisect-cli/src/runner/mod.rs', 'crates/bisect-cli/src/runner/support.rs',
                      'crates/bisect-cli/src/demographics.rs', 'crates/bisect-runner/src/state_args.rs']],
                  source_files=[pin(p) for p in sorted(sources.iterdir())], outcomes=outcomes,
                  scope=f'Actual retained native CLI observations, eleven four-unit {args.engine} nway attempts. Nine succeed; two malformed/existing CVAP targets fail during the separate audit-sidecar demographic loader. Availability checks CWD-relative raw native name; the later report loader normalizes the name and tries VAP/CVAP candidates independently of the requested population-source. Successful graph assignments, units and population vectors match total; valid CSV adds report demographics even when total is requested, and requested strings remain manifest metadata. The caller must provide a native build supporting the requested backend; this producer does not authenticate its build features. This is native behavior evidence, not a browser implementation, a population-vector switch, full flag parity or reproducible-build attestation.')
    (output / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(output, flush=True)


if __name__ == '__main__':
    main()
