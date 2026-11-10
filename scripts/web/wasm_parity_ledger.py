"""Validate structured inventory claims. This checks evidence, never executes it.

Default mode permits explicitly unresolved rows. --full-parity requires every
parent and required child contract to close. --audit-full-parity lists all
remaining closure conditions after validating the current inventories. No mode counts declarations
as executable support. Paths are repository-relative; hashes identify bytes,
not authorship or certification.
"""
import argparse
import hashlib
import json
import lzma
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[2]
INVENTORIES = ('docs/specs/wasm-capability-inventory.json',
               'docs/specs/wasm-engine-option-inventory.json')
SOURCE_CATALOGS = (
    ('docs/specs/wasm-practitioner-actions.json', 'bisect-practitioner-actions-v1'),
    ('docs/specs/wasm-fetch-source-coverage.json', 'bisect-fetch-source-coverage-v1'),
)
MAX_ENCODED_WASM_BYTES = 64 * 1024 * 1024
WASM_HEADER = b'\x00asm\x01\x00\x00\x00'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def file_ref(root, ref):
    path = ref.get('path', '')
    relative = PurePosixPath(path)
    require(path and not relative.is_absolute() and '\\' not in path and
            ':' not in path and '..' not in relative.parts, 'Unsafe evidence path')
    resolved = (root / path).resolve()
    require(resolved.is_relative_to(root.resolve()), 'Evidence escaped repository')
    require(resolved.is_file(), 'Missing evidence: ' + path)
    with resolved.open('rb') as source:
        require(hashlib.file_digest(source, 'sha256').hexdigest() == ref.get('sha256'),
                'Stale evidence: ' + path)
    return resolved


def validate_build_artifact(root, ref, kind):
    """Bind both transport bytes and the exact decoded WASM without executing it."""
    require(isinstance(ref, dict), 'Invalid build artifact: ' + kind)
    artifact = file_ref(root, ref)
    if 'encoding' not in ref:
        return
    encoding = ref['encoding']
    require(kind == 'wasm' and isinstance(encoding, dict) and
            set(encoding) == {'codec', 'decoded_sha256', 'decoded_byte_length'} and
            encoding['codec'] == 'xz', 'Unsupported build artifact encoding')
    raw_hash, remaining = encoding['decoded_sha256'], encoding['decoded_byte_length']
    require(isinstance(raw_hash, str) and re.fullmatch('[a-f0-9]{64}', raw_hash) and
            type(remaining) is int and len(WASM_HEADER) <= remaining <= MAX_ENCODED_WASM_BYTES and
            0 < artifact.stat().st_size <= MAX_ENCODED_WASM_BYTES, 'Invalid encoded WASM limits or identity')
    reader = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=MAX_ENCODED_WASM_BYTES)
    digest = hashlib.sha256()
    header = b''
    try:
        with artifact.open('rb') as source:
            while not reader.eof:
                chunk = source.read(1024 * 1024) if reader.needs_input else b''
                require(not reader.needs_input or bool(chunk), 'Truncated encoded WASM')
                block = reader.decompress(chunk, max_length=min(1024 * 1024, remaining + 1))
                require(len(block) <= remaining, 'Encoded WASM exceeds declared length')
                if len(header) < len(WASM_HEADER):
                    header += block[:len(WASM_HEADER) - len(header)]
                digest.update(block)
                remaining -= len(block)
            require(not reader.unused_data and not source.read(1), 'Trailing encoded WASM bytes')
    except lzma.LZMAError as error:
        raise ValueError('Invalid or excessive encoded WASM stream') from error
    require(not remaining and digest.hexdigest() == raw_hash, 'Decoded WASM identity differs')
    require(header == WASM_HEADER, 'Decoded build artifact is not a WASM module')


def pin(root, path):
    return {'path': path, 'sha256': hashlib.sha256((root / path).read_bytes()).hexdigest()}


def validate_current_source_catalogs(root):
    """Check current source identities, without refreshing or accepting evidence."""
    for relative, schema in SOURCE_CATALOGS:
        path = root / relative
        require(path.is_file(), 'Missing current source catalog: ' + relative)
        data = json.loads(path.read_text(encoding='utf-8'))
        require(isinstance(data, dict) and data.get('schema_version') == schema,
                'Unknown current source catalog: ' + relative)
        refs = data.get('authority')
        require(isinstance(refs, list) and refs, 'Missing current source authority: ' + relative)
        seen = set()
        for ref in refs:
            require(isinstance(ref, dict) and isinstance(ref.get('path'), str),
                    'Invalid current source reference: ' + relative)
            require(ref['path'] not in seen, 'Duplicate current source reference: ' + ref['path'])
            seen.add(ref['path'])
            source = file_ref(root, ref)
            require(type(ref.get('byte_length')) is int and ref['byte_length'] >= 0 and
                    source.stat().st_size == ref['byte_length'],
                    'Current source byte length differs: ' + ref['path'])
        rows = data.get('rows')
        require(isinstance(rows, list) and rows, 'Missing current source mappings: ' + relative)
        for row in rows:
            require(isinstance(row, dict), 'Invalid current source mapping: ' + relative)
            if schema == 'bisect-practitioner-actions-v1':
                dispatch = row.get('native_dispatch')
                require(isinstance(dispatch, dict), 'Missing current native dispatch: ' + relative)
                referenced = [row.get('native_declaration'), dispatch.get('path')]
                groups = ('native_kernel_sources', 'browser_sources', 'browser_rust_sources')
            else:
                referenced = []
                groups = ('native_authority', 'browser_sources', 'consumer_sources')
            for group in groups:
                require(group in row or group == 'browser_rust_sources',
                        'Missing current source mapping group: ' + group)
                names = row.get(group, [])
                require(isinstance(names, list), 'Invalid current source mapping group: ' + group)
                referenced.extend(names)
            for name in referenced:
                require(isinstance(name, str) and name in seen,
                        'Unpinned current source mapping: ' + str(name))


def validate_bundle(root, contract):
    comparison = contract.get('comparison')
    require(isinstance(comparison, dict), 'Execution comparison must be explicitly versioned before closure')
    require(type(comparison.get('version')) is int and comparison['version'] > 0,
            'Execution comparison requires a positive integer version')
    mode = comparison.get('mode')
    require(isinstance(mode, str) and mode and mode == mode.strip(),
            'Execution comparison requires an explicit nonempty mode')
    unfinished = ('contract-not-yet-finalized', 'pending', 'unfinalized', 'unreviewed', 'not-run', 'todo', 'tbd')
    require(mode.casefold() not in unfinished and not mode.casefold().startswith('pending-'),
            'Execution comparison is not finalized')
    require(isinstance(comparison.get('allowlist'), list),
            'Execution comparison requires an explicit difference allowlist')
    bundle_path = file_ref(root, contract['acceptance_bundle'])
    bundle = json.loads(bundle_path.read_text(encoding='utf-8'))
    require(bundle.get('schema') == 'bisect-parity-acceptance-v1', 'Unknown acceptance schema')
    for key in ('contract_id', 'scope', 'comparison', 'backend', 'claim_kind'):
        require(bundle.get(key) == contract[key], 'Bundle contract mismatch: ' + key)
    require(bundle.get('status') == 'passed', 'Evidence was not run or failed')
    require(bundle.get('outcome') in ('accepted-output', 'expected-native-failure'),
            'Execution and candidate acceptance must be distinguished')
    if contract.get('successful_support'):
        require(bundle['outcome'] == 'accepted-output', 'Native failure is not successful support')
    require(bundle.get('reference_kind') in ('native-cli', 'independent-native-kernel'),
            'Shared adapter/inspection alone cannot close execution')
    reference_kinds = comparison.get('reference_kinds')
    if reference_kinds is not None:
        require(isinstance(reference_kinds, list) and reference_kinds and
                all(kind in ('native-cli', 'independent-native-kernel') for kind in reference_kinds) and
                len(reference_kinds) == len(set(reference_kinds)), 'Invalid comparison native authority declaration')
        require(bundle['reference_kind'] in reference_kinds, 'Missing comparison-required native authority')
    require(contract['claim_kind'] in ('kernel', 'workflow'), 'Inspection cannot close execution')
    require(bundle.get('browser_exercised') is True, 'Browser action not exercised')
    require(bundle.get('reproducible_fixture') is True, 'Fixture is not reproducible')
    for key in ('source_files', 'test_sources', 'fixture_inputs', 'raw_outputs', 'comparison_results'):
        require(isinstance(bundle.get(key), list) and bundle[key], 'Missing evidence group: ' + key)
        for ref in bundle[key]:
            file_ref(root, ref)
    require(contract['native_consumer'] in bundle['source_files'], 'Native consumer not bound to evidence')
    require(contract['browser_action']['source'] in bundle['source_files'], 'Browser source not bound to evidence')
    require(any(not ref['path'].startswith(('target/', 'dist/')) for ref in bundle['fixture_inputs']),
            'Generated target outputs alone are not a reproducible fixture')
    require(bundle.get('checks') and all(check.get('status') == 'passed' for check in bundle['checks']),
            'Required checks include failed or not-run rows')
    required_checks = comparison.get('required_check_ids')
    if required_checks is not None:
        require(isinstance(required_checks, list) and required_checks and
                all(isinstance(name, str) and name and name == name.strip() for name in required_checks) and
                len(required_checks) == len(set(required_checks)), 'Invalid comparison required stage declaration')
        check_ids = [check.get('id') for check in bundle['checks']]
        require(all(check_ids.count(name) == 1 for name in required_checks),
                'Missing or repeated comparison-required stage checks')
    for artifact in ('native_executable', 'wasm'):
        validate_build_artifact(root, bundle['build'][artifact], artifact)
    profiles = bundle['build'].get('additional_profiles', {})
    require(isinstance(profiles, dict), 'Invalid additional build profiles')
    for profile in profiles.values():
        require(isinstance(profile, dict) and all(k in profile for k in ('native_executable', 'wasm')),
                'Missing additional profile artifacts')
        for artifact in ('native_executable', 'wasm'):
            validate_build_artifact(root, profile[artifact], artifact)
    for key in ('source_revision', 'toolchain', 'dependencies', 'build_settings',
                'browser_runtime', 'original_arguments', 'resolved_arguments',
                'seed', 'profile', 'integer_width', 'threading', 'floating_point'):
        require(key in bundle['build'], 'Missing build identity: ' + key)
    require(bundle['build'].get('backend') == contract['backend'], 'Substituted backend')


def parent_closure_issues(row):
    """The same ordered conditions used by the strict gate and the audit."""
    issues = []
    if row['evidence_state'] != 'verified':
        issues.append(('parent-evidence-unverified', 'Parent evidence has not been verified'))
    if row['implementation_state'] != 'implemented' or row['classification'] == 'unreconciled':
        issues.append(('parent-implementation-unresolved', 'Unreconciled/unimplemented parent cannot close'))
    if not row['native_authority'].get('consumer'):
        issues.append(('missing-native-consumer', 'Declaration alone is not a native consumer'))
    required = [c for c in row['contracts'] if c['required_for_full_parity']]
    if not required:
        issues.append(('missing-required-contracts', 'Required contracts are unresolved or not-run'))
    elif any(c['evidence_state'] != 'verified' for c in required):
        issues.append(('required-contracts-unverified', 'Required contracts are unresolved or not-run'))
    if row['open_issue']:
        issues.append(('open-issue', 'Parent still has an unresolved issue'))
    return issues


def validate_entry(root, entry, full=False):
    row = entry.get('parity_record', {})
    require(row.get('schema') == 'bisect-parity-record-v1', 'Missing structured parity record')
    for key in ('assignee', 'review_lenses', 'priority', 'dependencies', 'native_authority',
                'implementation_state', 'evidence_state', 'classification', 'contracts', 'open_issue'):
        require(key in row, 'Missing record field: ' + key)
    require(row['assignee'] and row['review_lenses'], 'Missing accountable owner/lenses')
    require(row['priority'] in ('P0', 'P1', 'P2', 'P3'), 'Unknown priority')
    require(row['implementation_state'] in ('unreviewed', 'missing', 'partial', 'implemented', 'shared-limitation'),
            'Unknown implementation state')
    require(row['evidence_state'] in ('unreviewed', 'not-run', 'partial', 'verified'), 'Unknown evidence state')
    require(row['classification'] in ('unreconciled', 'missing-behavior', 'browser-adaptation',
                                     'verification-gap', 'shared-limitation'), 'Unknown gap classification')
    authority = row['native_authority']
    file_ref(root, authority['declaration'])
    if authority.get('consumer'):
        file_ref(root, authority['consumer'])
    for ref in authority.get('related_sources', []):
        file_ref(root, ref)
    require(authority['declaration']['path'] == entry['source'], 'Declaration does not identify this row')
    seen = set()
    for contract in row['contracts']:
        identifier = contract['contract_id']
        require(identifier.startswith(entry['id'] + '/') and identifier not in seen,
                'Unstable/duplicate child contract ID')
        seen.add(identifier)
        for key in ('scope', 'comparison', 'backend', 'claim_kind', 'required_for_full_parity',
                    'implementation_state', 'evidence_state', 'browser_action', 'native_consumer'):
            require(key in contract, 'Missing child contract field: ' + key)
        require(type(contract['required_for_full_parity']) is bool, 'Required flag must be Boolean')
        require(contract['implementation_state'] in ('missing', 'partial', 'implemented', 'shared-limitation'),
                'Unknown child implementation state')
        require(contract['evidence_state'] in ('not-run', 'partial', 'verified'), 'Unknown child evidence state')
        require(contract['claim_kind'] in ('inspection', 'kernel', 'workflow'), 'Unknown claim kind')
        if contract.get('native_consumer'):
            file_ref(root, contract['native_consumer'])
        if action := contract.get('browser_action'):
            file_ref(root, action['source'])
            for ref in action.get('related_sources', []):
                file_ref(root, ref)
        for ref in contract.get('candidate_test_sources', []):
            file_ref(root, ref)
        verified = contract['evidence_state'] == 'verified'
        if verified:
            require(contract['implementation_state'] == 'implemented', 'Verified unimplemented behavior')
            file_ref(root, contract['native_consumer'])
            action = contract['browser_action']
            require(action and action.get('operation') and action.get('ui_hook'), 'Missing browser action')
            file_ref(root, action['source'])
            validate_bundle(root, contract)
    if row['evidence_state'] == 'verified' or full:
        issues = parent_closure_issues(row)
        require(not issues, issues[0][1] if issues else '')
    else:
        require(row['open_issue'], 'Unresolved record needs an explicit issue')


def validate_inventory(root, data, full=False):
    require(data.get('parity_ledger', {}).get('schema') == 'bisect-parity-ledger-v1', 'Missing ledger policy')
    seen = set()
    for entry in data['entries']:
        require(entry['id'] not in seen, 'Duplicate inventory ID')
        seen.add(entry['id'])
        try:
            validate_entry(root, entry, full)
        except (KeyError, TypeError, ValueError) as error:
            raise ValueError(entry['id'] + ': ' + str(error)) from error


def audit_inventory(root, data):
    """List missing closure conditions only after all present evidence validates."""
    validate_inventory(root, data)
    blocked = []
    for entry in data['entries']:
        row = entry['parity_record']
        issues = parent_closure_issues(row)
        if not issues:
            continue
        children = []
        for c in row['contracts']:
            if not c['required_for_full_parity'] or c['evidence_state'] == 'verified':
                continue
            scope, comparison = c['scope'], c['comparison']
            children.append({
                'id': c['contract_id'], 'backend': c['backend'], 'claim_kind': c['claim_kind'],
                'implementation': c['implementation_state'], 'evidence': c['evidence_state'],
                'remaining': scope.get('remaining') if isinstance(scope, dict) else None,
                'comparison_mode': comparison.get('mode') if isinstance(comparison, dict) else None,
                'native_consumer': (c.get('native_consumer') or {}).get('path'),
                'browser_operation': (c.get('browser_action') or {}).get('operation'),
                'test_sources': [r['path'] for r in c.get('candidate_test_sources', [])],
            })
        blocked.append({
            'id': entry['id'], 'priority': row['priority'], 'dependencies': row['dependencies'],
            'classification': row['classification'], 'implementation': row['implementation_state'],
            'evidence': row['evidence_state'], 'declaration': entry['source'],
            'native_consumer': (row['native_authority'].get('consumer') or {}).get('path'),
            'conditions': [{'code': code, 'message': message} for code, message in issues],
            'open_issue': row['open_issue'], 'unverified_required_contracts': children,
        })
    return blocked


def initialize(root, data):
    """Add pending records only. Never infer execution or erase existing review."""
    data.setdefault('parity_ledger', {
        'schema': 'bisect-parity-ledger-v1',
        'acceptance_owner': 'BISECT maintainer',
        'historical_fields': 'operation/ui/verification/review/evidence/limitation retain historical notes; parity_record governs reconciled claims only.',
        'milestone': 'WP-01 in progress; pending records are not reconciled support claims',
    })
    for entry in data['entries']:
        entry.setdefault('parity_record', {
            'schema': 'bisect-parity-record-v1',
            'assignee': 'Codex implementation; BISECT maintainer acceptance',
            'review_lenses': ['BENCHMARK', 'COVENANT'],
            'priority': 'P1' if entry['id'] in ('bisect.Run', 'bisect.State', 'bisect.States') else 'P2',
            'dependencies': ['WP-01'],
            'native_authority': {'declaration': pin(root, entry['source']), 'consumer': None},
            'implementation_state': 'unreviewed',
            'evidence_state': 'unreviewed',
            'classification': 'unreconciled',
            'contracts': [],
            'open_issue': 'WP-01: trace actual consumer, split mixed scopes, reconcile historical notes and bind reproducible native/browser acceptance evidence.',
        })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--initialize', action='store_true', help='Add pending records; does not refresh stale identities')
    mode.add_argument('--full-parity', action='store_true', help='Require every required execution contract to close')
    mode.add_argument('--audit-full-parity', action='store_true', help='Print a JSON list of unresolved closure conditions')
    args = parser.parse_args()
    # Coverage comes from the actual native enums/fields, not an editable count.
    # Removing an unsupported parent must not make a release claim pass.
    for scanner in ('wasm_inventory.py', 'wasm_engine_option_inventory.py', 'wasm_native_consumption.py'):
        arguments = [] if scanner == 'wasm_native_consumption.py' else ['--check']
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/web' / scanner), *arguments], cwd=ROOT,
                                stdout=sys.stderr if args.audit_full_parity else None)
        if result.returncode:
            raise SystemExit('Source inventory drift blocks parity validation')
    count = 0
    inventories = []
    try:
        validate_current_source_catalogs(ROOT)
        for path in INVENTORIES:
            raw = (ROOT / path).read_bytes()
            data = json.loads(raw)
            if args.initialize:
                initialize(ROOT, data)
                (ROOT / path).write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
            if args.audit_full_parity:
                blocked = audit_inventory(ROOT, data)
                inventories.append({'path': path, 'sha256': hashlib.sha256(raw).hexdigest(),
                                    'parent_records': len(data['entries']), 'blocked_parents': blocked})
            else:
                validate_inventory(ROOT, data, args.full_parity)
            count += len(data['entries'])
    except (KeyError, TypeError, ValueError) as error:
        raise SystemExit(str(error)) from error
    if args.audit_full_parity:
        print(json.dumps({'schema_version': 'bisect-parity-closure-audit-v1',
                          'ledger_conditions_satisfied': not any(i['blocked_parents'] for i in inventories),
                          'scope': 'Validated ledger closure conditions only; no runtime execution or release checks.',
                          'inventories': inventories}, indent=2, ensure_ascii=True))
        return
    print(f'{count} structured records valid; ' + ('full parity gate passed' if args.full_parity else
          'unresolved rows permitted; no execution or full-parity claim'))


if __name__ == '__main__':
    main()
