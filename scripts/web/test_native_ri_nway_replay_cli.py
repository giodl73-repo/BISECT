"""Actual seed-42 NWay State and replay-native-state on the retained Rhode Island tract graph."""
import argparse
import base64
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile

from test_native_ri_replay_cli import FIXTURE, ROOT, load, read_tree, sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--expected-backend', choices=['metis-core', 'c-ffi'], required=True)
    args = parser.parse_args()
    cli = args.cli.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='native-ri-replay-', dir=args.output.resolve()))
    fixture = load(FIXTURE)
    assert fixture['schema_version'] == 'bisect-retained-ri-tract-graph-v1'
    inputs = out / 'inputs'
    base = 'V3/data/2020/adjacency/ri_adjacency_2020'
    assert {r['path'] for r in fixture['files']} == {base + s for s in (
        '.adj.bin', '.pkl', '_centroids.json', '_geoids.json')}
    for r in fixture['files']:
        raw = base64.b64decode(r['base64'], validate=True)
        assert len(raw) == r['bytes'] and sha(raw) == r['sha256']
        p = inputs / r['path']
        p.parent.mkdir(parents=True, exist_ok=True)
        with p.open('xb') as f:
            f.write(raw)
    manifest = load(ROOT / 'data/manifest.json')
    manifest.update(local_outputs_dir=str(inputs), local_data_dir=str(out / 'data'))
    (out / 'data').mkdir()
    (out / 'source-manifest.json').write_text(json.dumps(manifest), encoding='utf8')
    env = dict(os.environ, BISECT_MANIFEST=str(out / 'source-manifest.json'))
    adj = inputs / (base + '.adj.bin')
    report = {'status': 'running', 'native': {'path': str(cli), 'sha256': sha(cli.read_bytes())},
              'fixture_sha256': sha(FIXTURE.read_bytes()), 'calls': {}, 'full_parity': False}

    def run(name, argv, cwd):
        p = subprocess.run([cli, *argv], cwd=cwd, env=env, capture_output=True, timeout=180)
        (out / (name + '.stdout')).write_bytes(p.stdout)
        (out / (name + '.stderr')).write_bytes(p.stderr)
        report['calls'][name] = dict(argv=argv, cwd=str(cwd), exit=p.returncode,
                                     stdout_sha256=sha(p.stdout), stderr_sha256=sha(p.stderr))
        return p

    def literal(path):
        d = load(path)['plan']
        return dict(zip(d['units']['unit_ids'], (a + 1 for a in d['assignment']), strict=True))

    try:
        state = out / 'state'
        state.mkdir()
        dest = state / 'outputs/v1'
        # Both native builds receive the same public flags. The recorded effective
        # backend depends on which native METIS runtime was compiled in.
        command = ['state', '--state', 'RI', '--year', '2020', '--chamber', 'congressional', '--districts', '2',
                   '--label', 'OriginalRITract', '--version', 'v1', '--output-dir', str(dest), '--adjacency', str(adj),
                   '--metis-engine', 'metis-core', '--structure', 'nway', '--search', 'single',
                   '--seed', '42', '--balance-tolerance', '0.5', '--force', '--manifest']
        p = run('state', command, state)
        assert p.returncode == 0, p.stderr
        package = dest / '2020/plans/OriginalRITract'
        original = package / 'manifest.json'
        plan = package / 'plan.rplan'
        recipe = load(original)['native_replay_recipe']
        assert recipe['recursive_metis_backend'] == args.expected_backend and recipe['state']['districts'] == 2
        assert recipe['state']['algorithm']['metis']['seed'] == '42'
        native_plan, original_tree, input_tree = literal(plan), read_tree(package), read_tree(inputs)
        replay = out / 'replay'
        replay.mkdir()
        replay_dest = replay / 'results'
        cmd = ['replay-native-state', '--manifest', str(original), '--output-dir', str(replay_dest), '--label', 'NativeReplay']
        p = run('native-replay', cmd, replay)
        assert p.returncode == 0, p.stderr
        replay_plan, = replay_dest.rglob('plan.rplan')
        assert literal(replay_plan) == native_plan
        keep = adj.read_bytes()
        # Alter only this fixture's copy. A rejected replay must leave no target.
        try:
            adj.write_bytes(keep[:-1] + bytes([keep[-1] ^ 1]))
            bad = out / 'changed-source'
            bad.mkdir()
            p = run('changed-source', [*cmd[:4], str(bad / 'results'), '--label', 'Changed'], bad)
            assert p.returncode == 1 and b'Native replay source changed or is unavailable' in p.stderr, p.stderr
            assert not (bad / 'results').exists()
        finally:
            adj.write_bytes(keep)
        assert read_tree(package) == original_tree and read_tree(inputs) == input_tree
        assert keep[:4] == b'RADJ'
        version, n, _ = struct.unpack_from('<III', keep, 4)
        assert version == 3 and n == fixture['units'] == len(native_plan)
        populations = struct.unpack_from(f'<{n}q', keep, 16)
        assert sum(populations) == fixture['population']
        adjacency, off = [], 16 + n * 8
        for _ in range(n):
            count, = struct.unpack_from('<I', keep, off)
            adjacency.append(struct.unpack_from(f'<{count}I', keep, off + 4))
            off += 4 + count * 4
        ids = load(inputs / (base + '_geoids.json'))
        assert set(ids.values()) == set(native_plan)
        district_populations = []
        for d in (1, 2):
            members = {i for i in range(n) if native_plan[ids[str(i)]] == d}
            q = [min(members)]
            seen = set(q)
            for i in q:
                for j in adjacency[i]:
                    if j in members and j not in seen:
                        seen.add(j)
                        q.append(j)
            assert seen == members
            pop = sum(populations[i] for i in members)
            assert abs(pop * 2 - sum(populations)) * 1000 <= sum(populations) * 5
            district_populations.append(pop)
        report.update(status='passed-actual-state-and-replay-native-on-original-ri-tracts',
                      original_manifest=str(original), original_manifest_sha256=sha(original.read_bytes()),
                      original_plan_sha256=sha(plan.read_bytes()), native_replay_plan=str(replay_plan),
                      effective_backend=recipe['recursive_metis_backend'], seed='42',
                      source_graph_sha256=recipe['source_graph_sha256'], assignments=native_plan,
                      district_populations=district_populations, changed_source_no_output=True,
                      original_root_unchanged=True, input_root=str(inputs))
    except BaseException as error:
        report.update(status='failed', failure=repr(error))
        raise
    finally:
        (out / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(out)


if __name__ == '__main__':
    main()
