"""Source-backed command/TUI workflow inventory; never runtime acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from wasm_native_consumption import code_mask

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / 'docs/specs/wasm-practitioner-actions.json'
MAIN = 'crates/bisect-cli/src/main.rs'
TUI = 'crates/bisect-tui/src/main.rs'

# Reviewed primary operation mappings, rather than substring matches in historical prose.
OPERATIONS = {
    'Run': 'inspect-native-run-command run-native-run-job run-state-batch',
    'State': 'inspect-native-state-command run-native-state-job run-native-selected-state-source run-state-batch prepare-fresh-native-state-run run-fresh-native-state',
    'States': 'inspect-native-states-command run-native-states-job run-state-batch',
    'Fetch': 'plan-fetch-sources prepare-fetch-archive-package prepare-fetch-data-package',
    'FletchSources': 'inspect-fletch-sources',
    'FletchCacheIndex': 'inspect-fletch-cache verify-fletch-cache-files stage-fletch-cache-source merge-fletch-cache-manifest',
    'Analyze': 'analyze-plan-package analyze-plan-election analyze-bloc-voting',
    'Compare': 'compare-plans render-report-comparison summarize-plan-labels',
    'Map': 'render-native-district-map render-native-national-map prepare-native-map-tiger render-native-national-map-batch',
    'Aggregate': 'aggregate-native-analysis aggregate-national-analysis', 'Validate': 'validate-native-rplan',
    'Migrate': 'inspect-native-migration-base stage-native-migration-base migrate-legacy-plan',
    'Suite': 'assemble-plan-suite validate-suite-nesting analyze-suite-stability',
    'Report': 'assemble-commission-report assemble-commission-audit prepare-plan-report-manifest',
    'Export': 'export-rplan-interchange export-native-plan-folder',
    'Import': 'import-plan-interchange import-native-plan', 'Improve': 'improve-rplan-text',
    'Exact': 'solve-certified-tree-context solve-certified-canonical-context solve-certified-discovery-context compile-certified-proof-models',
    'Ceremony': 'start-ceremony announce-ceremony halt-ceremony verify-ceremony',
    'Civic': 'ingest-civic-coi inspect-civic-coi inspect-native-civic-conflicts inspect-civic-manifests ingest-candidate-race',
    'Research': 'validate-research-ensemble diagnose-partition-traces',
    'Policy': 'inspect-location-policy', 'Doctor': 'diagnose-location-inputs diagnose-plan-artifacts verify-tutorial-files',
    'Verify': 'verify-native-output-workspace verify-manifest-prechecks verify-manifest-replay verify-manifest-certificate verify-rplan-package compare-verify-assignments',
    'IlpAudit': 'audit-ilp-artifacts', 'Sweep': 'plan-sweep',
    'Tui': 'inspect-tui-plan-library inspect-tui-session run-tui-configured run-accepted-state-workflow compare-tui-plans verify-tui-manifest diagnose-tui-location',
    'Config': 'create-algorithm-config validate-algorithm-config inspect-algorithm-config',
    'Build': 'run-label-build', 'Ls': 'inspect-label-registry', 'Show': 'inspect-label-registry',
    'Mv': 'rename-label-package', 'LabelVerify': 'verify-label-chain',
    'LabelAnalyze': 'run-label-analysis run-label-analysis-years reconcile-label-registry reconcile-label-package',
    'LabelReport': 'run-label-report run-label-report-years reconcile-label-registry reconcile-label-package',
    'LabelImport': 'import-label-assignments prepare-label-import-plan', 'LabelCompare': 'inspect-label-comparison',
    'Plan': 'inspect-plan-hub', 'Ensemble': 'sample-smc-context', 'Pareto': 'sample-pareto-context',
}
MODULES = dict(zip(OPERATIONS, [
    'runner/mod.rs', 'runner/mod.rs', 'runner/mod.rs', 'fetch.rs', 'fletch.rs', 'fletch.rs',
    'analyze.rs', 'compare.rs', 'map_cmd.rs', 'aggregate.rs', 'validate.rs', 'migrate.rs',
    'suite.rs', 'report_cmd.rs', 'export_cmd.rs', 'import_cmd.rs', 'improve_cmd.rs',
    'exact_cmd.rs', 'ceremony_cmd.rs', 'civic.rs', 'research.rs', 'policy.rs', 'doctor.rs',
    'verify.rs', 'ilp_audit.rs', 'sweep.rs', 'plan_cmd.rs', 'algo_config.rs', 'build_cmd.rs',
    'label_cmd.rs', 'label_cmd.rs', 'label_cmd.rs', 'label_cmd.rs', 'analyze_label.rs',
    'analyze_label.rs', 'import_label.rs', 'import_label.rs', 'plan_cmd.rs', 'ensemble.rs', 'pareto_cmd.rs']))
MODULES['IlpAudit'] = 'lib.rs'
# Resolved replay is an explicit selected-byte browser adaptation of native IO.
OPERATIONS['ReplayNative'] = 'inspect-native-replay-recipe inspect-native-replay-sources prepare-native-recorded-replay run-native-recorded-replay'
MODULES['ReplayNative'] = 'native_replay.rs'
OTHER = {
    'suite.Draw': 'assemble-plan-suite', 'suite.Validate': 'validate-suite-nesting',
    'suite.Stability': 'analyze-suite-stability', 'config.New': 'create-algorithm-config',
    'config.Validate': 'validate-algorithm-config inspect-algorithm-config',
    'ceremony.Start': 'start-ceremony', 'ceremony.Announce': 'announce-ceremony',
    'ceremony.Halt': 'halt-ceremony', 'ceremony.Verify': 'verify-ceremony',
    'civic.Ingest': 'ingest-civic-coi', 'civic.Conflicts': 'inspect-native-civic-conflicts',
    'civic.List': 'inspect-civic-manifests', 'civic.Show': 'inspect-civic-manifests',
    'civic.AddCandidateRace': 'ingest-candidate-race', 'research.ValidateEnsemble': 'validate-research-ensemble',
    'rplan.Audit': 'audit-rplan-text', 'rplan.VerifyCertificate': 'verify-audit-text',
    'rcount.Verify': 'verify-count-files', 'rcount.ReplayAuditAlgorithms': 'replay-count-audits',
    'rcount.AggregateDistricts': 'aggregate-districts', 'rcount.ImportStatementCsv': 'import-statement-csv',
    'rcount.ImportNistCdfJson': 'import-nist-cdf-json', 'rcount.ImportRi2024Rep28Rla': 'import-ri2024-rep28-rla',
    'rhist.Verify': 'verify-history-files', 'tui.Home': 'inspect-tui-plan-library inspect-tui-session',
    'tui.Run': 'run-tui-configured', 'tui.Compare': 'compare-tui-plans',
    'tui.Verify': 'verify-tui-manifest', 'tui.Doctor': 'diagnose-tui-location',
}
TUI_ACTIONS = [
    ('home.run', "KeyCode::Char('r')", 'Session-default Run form', 'Home Run/session handoff', False),
    ('home.compare', "KeyCode::Char('c')", 'Visible selected label as Compare A', 'Library-to-Compare handoff', False),
    ('home.verify', "KeyCode::Char('v')", 'Verify form navigation', 'Library manifest/source Verify handoff', False),
    ('home.doctor', "KeyCode::Char('d')", 'Saved location/chamber and literal 2020', 'Explicit Doctor form/route; do not infer current law', False),
    ('home.selection', 'KeyCode::Up', 'Bounded Up/Down selected row', 'Idle/edit-guarded keyboard selection', False),
    ('home.sort', 'app.cycle_sort()', 'Seven native sorts preserving row position', 'Shared Rust sort, button and keyboard action', False),
    ('run.execute', 'spawn_bisect_state', 'Eight form fields delegate to State subprocess', 'Prepared TUI run plus separate original State workspace path', False),
    ('verify.execute', 'run_verify_subprocess', 'Native TUI always invokes verify --manifest --dry-run', 'Prechecks default; explicit replay is a separate browser action', False),
    ('compare.execute', 'compute_compare_result', 'Read hardcoded outputs/v1/2020/plans pair', 'Explicit two-folder retained sources', False),
    ('doctor.edit', 'Screen::Doctor', 'Edit location; r navigates to default Run form', 'Explicit Doctor/Run; no automatic partition', False),
    ('navigation.back', 'app.navigate_back()', 'Esc navigates history', 'Back route with busy/edit guards', False),
    ('glossary', 'app.show_glossary', 'Seven literal definitions, key closes modal', 'Accessible modal with close/focus restoration', False),
    ('palette.execute', 'fn execute_palette_command', 'run/compare/verify/doctor aliases navigate; unknown errors', 'Palette routes existing actions; no invented executor', False),
    ('palette.advertised-only', 'fn execute_palette_command', 'sort/filter/export/analyze/map/fetch/help suggestions are unimplemented by executor', 'Separate existing browser actions, not native palette parity', True),
    ('palette.quit', 'fn execute_palette_command', 'Palette quit/q is no-op; actual screen q exits', 'Browser tab close is not a native process exit', True),
    ('startup.configure', 'startup_configure', 'Configure/config flags stored without a native wizard consumer', 'Existing YAML template/validation/run remain separate', True),
    ('startup.session', 'session::load_session()', 'Session preferences, label selection and automatic plan discovery', 'Explicit saved preferences/selected sources; inert Open', False),
]


RANKED_MISSING_WORKFLOWS = []  # Prior bounded application shortlist delivered; broader audit remains open.



def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pin(name):
    path = ROOT / name
    return dict(path=name, sha256=sha(path), byte_length=path.stat().st_size)


def surface():
    text = (ROOT / 'web/lab/project.js').read_text(encoding='utf-8')
    project = set(re.findall(r"'([a-z][a-z0-9-]+)'", re.search(r'const operations = \[(.*?)\];', text, re.S)[1]))
    worker = set(re.findall(r"operation===?'([a-z][a-z0-9-]+)'", (ROOT / 'web/lab/wasm-worker.js').read_text(encoding='utf-8')))
    toolkit = (ROOT / 'crates/bisect-wasm/src/toolkit.rs').read_text(encoding='utf-8')
    names = re.findall(r'^    ([A-Z][A-Za-z0-9_]*)\s*[{(,]', toolkit[:toolkit.index('pub fn execute')], re.M)
    return project | worker | {re.sub(r'(?<!^)([A-Z])', r'-\1', name).lower() for name in names}


AGGREGATE_WORKSPACE_SOURCES = ['web/lab/native-aggregate-workspace.js', 'web/lab/selected-workspace-io.js', 'web/lab/native-aggregate-application-plan.js', 'web/lab/native-aggregate-application.js', 'web/lab/native-aggregate-application-journal.js', 'web/lab/native-aggregate-application-ui.js', 'web/lab/project.js', 'web/lab/workbench.html'] + ['web/lab/native-aggregate-effects.js', 'web/lab/native-aggregate-effect-plan.js', 'web/lab/native-aggregate-effect-application.js', 'web/lab/native-aggregate-effect-journal.js', 'web/lab/native-aggregate-effect-io.js', 'web/lab/native-aggregate-browser-workspaces.js', 'web/lab/native-aggregate-workspace-backup.js', 'web/lab/native-aggregate-workspace-inspection.js']
SHARED_BROWSER_PACKAGE_SOURCES = ['crates/bisect-web/build.rs', 'crates/bisect-web/src/input_assets.rs', 'crates/bisect-web/src/catalog.rs', 'web/lab/lab.js', 'scripts/web/test_native_asset_build.mjs', 'scripts/web/test_native_web_assets_browser.mjs', 'scripts/web/test_native_aggregate_selection.mjs', 'scripts/web/test_native_aggregate_workspace_browser.mjs', 'scripts/web/build_wasm.ps1', 'scripts/web/build_wasm_lab.ps1', 'scripts/web/build_cffi_preview.ps1', 'scripts/web/wasm_practitioner_inventory.py', 'scripts/web/fixtures/native-output-directory.mjs', 'scripts/web/workbench_project_open.mjs'] + ['scripts/web/test_native_aggregate_effects_suite.mjs', 'scripts/web/fixtures/native-aggregate-effects.mjs', 'scripts/web/fixtures/native-aggregate-browser-io.mjs', 'scripts/web/test_native_aggregate_effect_plan.mjs', 'scripts/web/test_native_aggregate_effect_application.mjs', 'scripts/web/test_native_aggregate_effect_recovery.mjs', 'scripts/web/test_native_aggregate_effect_recompute.mjs', 'scripts/web/test_native_aggregate_effect_io_browser.mjs', 'scripts/web/test_native_aggregate_effect_coordination_browser.mjs', 'scripts/web/test_native_aggregate_effect_persistence_browser.mjs', 'scripts/web/test_native_aggregate_effect_recovery_browser.mjs', 'scripts/web/test_native_aggregate_workspace_browser.mjs', 'scripts/web/test_native_aggregate_workspace_backup.mjs', 'scripts/web/test_native_cli_verify_replay.py', 'scripts/web/test_ilp_native_summary_paths.mjs', 'scripts/web/test_ilp_native_summary_paths_browser.mjs', 'scripts/web/test_census_2010_endpoints.mjs', 'scripts/web/test_census_2010_browser.mjs', 'scripts/web/fixtures/census-2010-official-endpoints.json', 'scripts/web/fixtures/census-2010-legacy-local-project.json', 'web/lab/census-recorded-endpoints.js', 'scripts/web/test_native_aggregate_workspace_backup_browser.mjs', 'scripts/web/test_native_aggregate_incomplete_workspace_browser.mjs', 'scripts/web/test_native_aggregate_workspace_inspection.mjs', 'web/lab/count-results.js', 'web/lab/reproducibility-files.js', 'web/lab/fletch-cache-backup-zip.js']
SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/check_build_gate_dependencies.py', 'scripts/web/test_build_gate_dependencies.py']
SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_config_build_wasm.mjs', 'scripts/web/fixtures/config-build-retry-inputs.json']
SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_prepared_engine.mjs']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/wasm_inventory.py', 'scripts/web/test_wasm_inventory.py']
SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_native_state_extraction.py', 'scripts/web/test_native_state_extraction_transitions.py', 'docs/specs/wasm-native-state-source-baseline.json', 'docs/specs/wasm-native-state-source-transitions.json', 'signals/roles/check/wasm-native-state-source-transition-roles-check-2026-11-02.md']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/check_fine_unit_map_package.py', 'scripts/web/test_fine_unit_map.mjs', 'scripts/web/test_fine_unit_map_browser.mjs', 'scripts/web/test_native_fine_maps.py', 'scripts/web/fixtures/fine-unit-map.mjs', 'scripts/web/fixtures/fine-unit-map/corpus.json', 'scripts/web/fixtures/fine-unit-map/manifest.json', 'signals/roles/check/wasm-fine-unit-map-workflow-roles-check-2026-11-02.md']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/check_native_replay_export_package.py', 'scripts/web/test_native_replay_export.mjs', 'scripts/web/test_native_replay_export_browser.mjs', 'web/lab/native-recorded-replay-results.js', 'crates/bisect-cli/examples/native_map_reference.rs', 'crates/bisect-cli/examples/fine_context_map_reference.rs', 'docs/specs/wasm-recorded-fine-replay-export.md', 'signals/roles/check/wasm-recorded-fine-replay-export-roles-check-2026-11-02.md', 'scripts/web/fixtures/native-recorded-replay.mjs', 'scripts/web/fixtures/native-recorded-replay/area-empty-refusal.json', 'scripts/web/fixtures/native-recorded-replay/area-reordered.json', 'scripts/web/fixtures/native-recorded-replay/c-native64-bg.json', 'scripts/web/fixtures/native-recorded-replay/c-nway-effective-c.json', 'scripts/web/fixtures/native-recorded-replay/core-native64-bg.json', 'scripts/web/fixtures/native-recorded-replay/core-nway-empty.json', 'scripts/web/fixtures/native-recorded-replay/manifest.json', 'scripts/web/fixtures/native-recorded-replay/native64-metadata.json']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_native_cli_unused.py', 'scripts/web/native_cli_state_observables.py', 'signals/roles/check/wasm-max-u64-counterfactual-roles-check-2026-11-02.md']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/check_native_ordinary_context_package.py', 'scripts/web/test_native_ordinary_context.mjs', 'scripts/web/test_native_ordinary_context_browser.mjs', 'scripts/web/fixtures/native-ordinary-context.mjs', 'scripts/web/fixtures/native-ordinary-recorded-replay.json', 'docs/specs/wasm-recorded-ordinary-context.md', 'signals/roles/check/wasm-recorded-ordinary-context-roles-check-2026-11-02.md', 'scripts/web/fixtures/native-ordinary-recorded-replay/c-state-baseline.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/c-state-dpi.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/c-state-objtype.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/c-state-print_only.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/c-state-target_mm_districts.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/cffi-nway-original.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/core-state-baseline.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/core-state-dpi.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/core-state-objtype.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/core-state-print_only.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/core-state-target_mm_districts.json', 'scripts/web/fixtures/native-ordinary-recorded-replay/manifest.json']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/fixtures/native-imported-recorded-replay.json', 'scripts/web/fixtures/native-unit-identity.mjs', 'scripts/web/test_native_unit_identity.mjs', 'scripts/web/test_native_unit_identity_browser.mjs', 'scripts/web/check_native_unit_identity_package.py', 'docs/specs/wasm-native-unit-identity.md']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_engine_option_inventory_history.py', 'signals/roles/check/wasm-native-unit-identity-roles-check-2026-11-03.md']

SHARED_BROWSER_PACKAGE_SOURCES += ['crates/bisect-runner/src/native_plan_context.rs', 'crates/bisect-wasm/src/fresh_native_state.rs', 'web/lab/fresh-native-state-panel.js', 'scripts/web/fixtures/fresh-native-state.mjs', 'scripts/web/test_fresh_native_state.mjs', 'scripts/web/test_fresh_native_state_browser.mjs', 'scripts/web/fixtures/native-plan-context-physical-v1/absent-index-coi.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/absent-index-coi.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/absent-index.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/absent-index.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/block-group-native-spelling.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/block-group-native-spelling.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/block-group-native-spelling.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/block.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/block.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/block.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/county-ids.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/county-ids.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/county-ids.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/foreign-prefix-tract.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/foreign-prefix-tract.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/foreign-prefix-tract.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/manifest.json', 'scripts/web/fixtures/native-plan-context-physical-v1/mixed-imported.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/mixed-imported.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/mixed-imported.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/no-prefix-imported.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/no-prefix-imported.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/no-prefix-imported.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/prefix-imported.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/prefix-imported.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/prefix-imported.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/sparse-index-coi.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/sparse-index-coi.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/sparse-index-coi.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/sparse-index.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/sparse-index.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/sparse-index.geoids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/tract.adj.bin', 'scripts/web/fixtures/native-plan-context-physical-v1/tract.centroids.json', 'scripts/web/fixtures/native-plan-context-physical-v1/tract.geoids.json']

SHARED_BROWSER_PACKAGE_SOURCES += ['web/lab/fresh-native-state-session.js', 'scripts/web/test_fresh_native_state_project.mjs', 'scripts/web/test_fresh_native_state_project_browser.mjs']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/fixtures/fresh-native-multiscale.mjs', 'scripts/web/test_fresh_native_multiscale.mjs', 'scripts/web/test_fresh_native_multiscale_browser.mjs', 'scripts/web/fixtures/fresh-native-multiscale/c-native64-bg.json', 'scripts/web/fixtures/fresh-native-multiscale/core-native64-bg.json', 'scripts/web/fixtures/fresh-native-multiscale/manifest.json', 'scripts/web/fixtures/fresh-native-multiscale/tract-adaptive.json', 'scripts/web/fixtures/fresh-native-multiscale/tract-county.json']

SHARED_BROWSER_PACKAGE_SOURCES += ['web/lab/fresh-native-state-results.js', 'scripts/web/fixtures/fresh-native-followup-checks.mjs', 'scripts/web/test_fresh_native_followups_browser.mjs']

SHARED_BROWSER_PACKAGE_SOURCES += ['web/lab/fresh-native-state-retained-view.js', 'scripts/web/test_fresh_native_state_retained_view.mjs', 'scripts/web/capture_build_sources.py', 'scripts/web/test_capture_build_sources.py']

SHARED_BROWSER_PACKAGE_SOURCES += ['web/lab/fresh-native-state-auxiliary.js', 'scripts/web/test_fresh_native_auxiliary_browser.mjs', 'scripts/web/fixtures/fresh-native-auxiliary.json']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_config_publication_windows.mjs', 'scripts/web/fixtures/hold-read-share.ps1']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_fresh_native_absence_browser.mjs', 'scripts/web/test_fresh_native_absence_sources.mjs', 'scripts/web/fixtures/fresh-native-absence.json', 'scripts/web/fixtures/fresh-native-absence-failures.json']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/parity_evidence_archive.py', 'scripts/web/test_parity_evidence_archive.py', 'docs/specs/wasm-parity-child-evidence.lock.json']
SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_large_workbench_archive_preference.mjs']

SHARED_BROWSER_PACKAGE_SOURCES += ['scripts/web/test_fresh_native_backend_profile.mjs', 'scripts/web/test_fresh_native_backend_profile_browser.mjs', 'scripts/web/fixtures/fresh-native-backend-profile.json']

def initialize():
    if OUTPUT.exists():
        raise ValueError('Review/version the existing inventory; do not silently regenerate source acceptance')
    entries = json.loads((ROOT / 'docs/specs/wasm-capability-inventory.json').read_text(encoding='utf-8'))['entries']
    rows, files = [], {'crates/bisect-wasm/src/toolkit.rs', 'web/lab/project.js',
                      'web/lab/workbench.js', 'web/lab/workbench.html', 'web/lab/wasm-worker.js',
                      'web/lab/workbench-navigation.js'}
    for entry in entries:
        identifier = entry['id']
        family, variant = identifier.split('.')
        dispatch, anchor = entry['source'], 'Commands::' + variant
        if family == 'bisect':
            dispatch, kernels = MAIN, ['crates/bisect-cli/src/' + MODULES[variant]]
            if variant == 'IlpAudit':
                kernels += ['crates/bisect-runner/src/ilp_audit.rs']
            operations = OPERATIONS[variant].split()
        else:
            operations = OTHER[identifier].split()
            kernels = [entry['source']]
            if family in ['suite', 'config']:
                dispatch, anchor = MAIN, ('SuiteCommands' if family == 'suite' else 'ConfigCommands') + '::' + variant
                kernels = ['crates/bisect-cli/src/' + ('suite.rs' if family == 'suite' else 'algo_config.rs')]
            elif family in ['ceremony', 'civic', 'research']:
                anchor = {'ceremony': 'CeremonySubcommand', 'civic': 'CivicSubcommand', 'research': 'ResearchSubcommand'}[family] + '::' + variant
            elif family == 'tui':
                dispatch, anchor = TUI, 'Screen::' + variant
                kernels += ['crates/bisect-tui/src/runner.rs', 'crates/bisect-tui/src/screens/' + variant.lower() + '.rs']
        if identifier == 'bisect.Report':
            kernels += ['crates/bisect-report/src/report.rs','crates/bisect-report/src/report_package.rs','crates/bisect-report/src/report_assembly.rs','crates/bisect-report/src/html.rs','crates/bisect-report/templates/report.html.j2']
        text = (ROOT / dispatch).read_text(encoding='utf-8')
        offset = code_mask(text).index(anchor)
        gap = 'Mapped kernel/action exists; full current native/browser/workflow/capacity verification remains required.'
        priority, classification = 4, 'browser-adaptation'
        if variant in ['Run', 'State', 'States'] and family == 'bisect':
            gap = 'Preserve delivered workspace/reset/continuation; broader sources/layouts/backend/capacity and automatic output installation remain open.'
        if variant in ['Fetch', 'FletchSources', 'FletchCacheIndex']:
            gap = 'Preserve delivered paired/archive recovery; release/election adapters and full delay/force/streaming lifecycle remain open.'
        if identifier in ['bisect.Build', 'bisect.LabelAnalyze', 'bisect.LabelReport', 'bisect.LabelImport', 'bisect.Mv']:
            priority = 1
            gap = 'Reviewed live-root analysis/report application has a scoped delivery; Build filesystem effects remain separate; import and rename have bounded local deliveries.'
        if identifier == 'bisect.Report':
            priority = min(priority, 2)
            gap += ' Browser PDF/print delivery has a bounded native-HTML contract; external native tools, court/PDF conformance, other hosts and formal release acceptance remain separate.'
        if identifier == 'rcount.Verify':
            priority = 3
            gap = 'Reviewed transcript application has a scoped native/default/C/final-browser delivery; broader host/capacity and formal release acceptance remain open.'
        if identifier == 'suite.Draw':
            classification = 'shared-limitation'
            gap = 'Native CLI Draw passes empty assignments and performs no automatic chamber generation; supplied-plan assembly already exists.'
        authority = 'Computed checks/model/profile scope, not official certification, eligibility adjudication, recount decision or source authenticity.' if family in ['rcount', 'rhist', 'rplan', 'ceremony'] or variant == 'Exact' else 'Actual native behavior only; declaration/button/test counts do not establish complete execution or legal authority.'
        browser_sources = ['web/lab/workbench.js']
        if identifier in ['bisect.LabelAnalyze', 'bisect.LabelReport']:
            browser_sources += ['web/lab/label-repository-install.js','web/lab/label-repository-install-ui.js']
        if identifier in ['bisect.Report', 'bisect.LabelReport', 'bisect.Compare']:
            browser_sources += ['web/lab/report-print.js','web/lab/generated-report-files.js','web/lab/commission-preview.js','web/lab/commission-preview.html']
        if identifier == 'bisect.LabelReport':
            classification = 'browser-adaptation'
            kernels += ['crates/bisect-report/src/label_summary.rs']
            browser_sources += ['web/lab/native-label-summary.js','web/lab/label-report-files.js','web/lab/native-summary-application.js','web/lab/native-summary-application-ui.js']
            gap += ' Native summary HTML/JSON, native ordering and safe relative outputs now have scoped local delivery; reviewed ordered live application now also has scoped delivery; broader host/capacity/release acceptance remains open. Per-state commission reports and PDF convenience remain browser extensions.'
        if identifier == 'bisect.Build':
            browser_sources += ['web/lab/label-build-files.js','web/lab/label-build-application.js','web/lab/build-repository.js','web/lab/label-build-application-ui.js','web/lab/label-build-archive.js','web/lab/project.js']
            kernels += ['crates/bisect-cli/src/runner/mod.rs','crates/bisect-runner/src/balance_retry.rs']
            files.update(['crates/bisect-wasm/src/label_build.rs','crates/bisect-wasm/src/engine_build.rs','crates/bisect-wasm/src/config.rs','crates/bisect-wasm/src/toolkit.rs'])
            gap += ' Reviewed Build application now has scoped fresh native/default/C and final-package worker/OPFS delivery with verified inherited storage; broader source/host/lifecycle/capacity/formal acceptance remain open.'
        if identifier == 'bisect.LabelImport':
            kernels += ['crates/bisect-report/src/label_import.rs']
            files.add('crates/bisect-wasm/src/label_import.rs')
            browser_sources += ['web/lab/label-import-files.js','web/lab/label-import-application.js','web/lab/label-import-application-ui.js']
            gap += ' Reviewed import has scoped actual CLI/default/C/final-package worker/OPFS delivery and a verified inherited snapshot; broader host/API/capacity/native lifecycle and formal release acceptance remain open.'
        if identifier == 'bisect.Mv':
            kernels += ['crates/bisect-report/src/label_rename.rs']
            files.add('crates/bisect-wasm/src/label_rename.rs')
            browser_sources += ['web/lab/label-rename-application.js','web/lab/label-rename-application-ui.js']
            gap += ' Reviewed rename has scoped native/default/C/final-browser delivery and verified immutable evidence, including native case-alias safety. Broader host/API/capacity/whole-parent/formal release acceptance remain open.'
        if identifier == 'rcount.Verify':
            browser_sources += ['web/lab/count-transcript-install.js','web/lab/count-transcript-install-ui.js']
            kernels += ['crates/rcount-audit/src/lib.rs']
        if variant == 'Tui':
            browser_sources += ['web/lab/state-workflow-files.js','web/lab/state-run-batch.js','web/lab/state-batch-zip.js','web/lab/workbench-archive.js']
            kernels += ['crates/bisect-analysis/src/district_geometry.rs']
            files.add('crates/bisect-wasm/src/state_workflow.rs')
        if identifier in ('bisect.Analyze','bisect.Run','bisect.State','bisect.States','bisect.Tui','tui.Run'):
            browser_sources += ['web/lab/election-source-format.js','web/lab/election-source-format-ui.js']
        if identifier == 'bisect.Fetch':
            browser_sources += ['web/lab/census-recorded-endpoints.js']
        if identifier in ('bisect.Fetch','bisect.Run','bisect.State','bisect.States'):
            browser_sources += ['web/lab/adjacency-release.js','web/lab/adjacency-release-ui.js','web/lab/native-fetch-release-package.js','web/lab/native-fetch-install.js','web/lab/native-fetch-install-ui.js','web/lab/fetch-install-outcome-store.js','web/lab/installation-outcome-store.js','cloudflare/adjacency-release-bridge.mjs','cloudflare/source-worker.mjs','wrangler.cloudflare.jsonc']
        if identifier in ('bisect.State','bisect.Run','bisect.States','bisect.Tui'):
            browser_sources += ['web/lab/native-state-workflow.js','web/lab/native-state-job.js','web/lab/native-state-diagnostics.js','web/lab/partition-timing.js']
            files.add('crates/bisect-wasm/src/partition_timing.rs')
        if identifier == 'bisect.State':
            browser_sources += ['web/lab/fresh-native-state-panel.js','web/lab/fresh-native-state-session.js','web/lab/fresh-native-state-auxiliary.js']
            browser_sources += ['web/lab/native-report-source-inputs.js','web/lab/native-state-report-pipeline.js','web/lab/native-report-export-binding.js','web/lab/native-report-source-ui.js']
            files.update(['crates/bisect-wasm/src/state_workflow.rs','crates/bisect-wasm/src/analysis_package.rs'])
        if identifier in ('bisect.Run','bisect.State','bisect.States'):
            browser_sources += ['web/lab/native-adjacency-bundle.js','web/lab/native-adjacency-files.js']
        if identifier in ('bisect.Run','bisect.State','bisect.States'):
            browser_sources += ['web/lab/native-state-output-ui.js','web/lab/installation-outcome-store.js']
        if identifier == 'bisect.Run':
            browser_sources += ['web/lab/state-run-batch.js','web/lab/state-batch-zip.js','web/lab/workbench-archive.js']
        if identifier == 'bisect.Map':
            browser_sources += ['web/lab/national-map-batch.js','web/lab/map-export-files.js']
            kernels += ['crates/bisect-map/src/renderer.rs']
            files.add('crates/bisect-wasm/src/national_map_stream.rs')
        if family == 'tui' or variant in ['Tui', 'Plan']:
            browser_sources += ['web/lab/workbench-navigation.js']
        if identifier == 'bisect.Exact':
            browser_sources += ['web/lab/native-proof-page.js', 'web/lab/native-tree-page.js']
            gap += ' Existing tree/split UI and native plan export must not be described as absent; larger proof/runtime and CLI orchestration acceptance remain open.'
        rows.append(dict(id=identifier, native_declaration=entry['source'],
            native_dispatch=dict(path=dispatch, anchor=anchor, line=text.count('\n', 0, offset) + 1),
            native_kernel_sources=kernels, browser_operations=operations, browser_sources=browser_sources,
            browser_rust_sources=(['crates/bisect-wasm/src/partition_timing.rs'] if identifier in ('bisect.State','bisect.Run','bisect.States','bisect.Tui') else []) + (['crates/bisect-wasm/src/state_workflow.rs'] if variant=='Tui' else ['crates/bisect-wasm/src/label_build.rs','crates/bisect-wasm/src/engine_build.rs','crates/bisect-wasm/src/config.rs','crates/bisect-wasm/src/toolkit.rs'] if identifier=='bisect.Build' else ['crates/bisect-wasm/src/label_import.rs'] if identifier=='bisect.LabelImport' else ['crates/bisect-wasm/src/label_rename.rs'] if identifier=='bisect.Mv' else ['crates/bisect-wasm/src/label_report.rs','crates/bisect-wasm/src/native_label_summary.rs'] if identifier=='bisect.LabelReport' else []),
            source_review_state='source-reconciled', runtime_verification='not-rerun-by-inventory',
            implementation_state='partial', classification=classification, remaining=gap, priority=priority,
            assignee='coding agent; BISECT maintainer milestone acceptance', dependencies=['WP-01', 'WP-07'],
            comparison_contract='Preserve exact original bytes/arguments/integers; compare actual native outcomes and artifacts, then separately verify composed browser/filesystem actions.',
            authority_boundary=authority, full_parity_claim=False))
        if identifier == 'civic.Conflicts':
            rows[-1]['native_kernel_sources'] += ['crates/bisect-analysis/src/civic.rs']
            rows[-1]['browser_rust_sources'] = ['crates/bisect-wasm/src/native_civic_conflicts.rs','crates/bisect-wasm/src/toolkit.rs']
            rows[-1]['browser_sources'] += ['web/lab/workbench.html','web/lab/civic-files.js','web/lab/project.js','web/lab/count-results.js']
            rows[-1]['remaining'] = 'Selected normalized bytes and ordered labels have scoped native/worker evidence; broader package discovery, capacity and formal acceptance remain open.'
            files.update(rows[-1]['native_kernel_sources'] + rows[-1]['browser_rust_sources'] + rows[-1]['browser_sources'])
        if identifier == 'bisect.Aggregate':
            rows[-1]['browser_rust_sources'] = ['crates/bisect-wasm/src/native_aggregate.rs', 'crates/bisect-wasm/src/national.rs']
            rows[-1]['browser_sources'] += ['web/lab/native-aggregate-files.js', 'web/lab/native-aggregate-catalog.js'] + AGGREGATE_WORKSPACE_SOURCES
            files.update(rows[-1]['browser_rust_sources'] + rows[-1]['browser_sources'])
        if identifier == 'bisect.Map':
            rows[-1]['native_kernel_sources'].append('crates/bisect-cli/src/geometry.rs')
            rows[-1]['browser_rust_sources'] += ['crates/bisect-wasm/src/national_map_stream.rs','crates/bisect-wasm/src/native_map_tiger.rs','crates/bisect-wasm/src/native_tiger_assignment_map.rs']
            rows[-1]['browser_sources'] += ['web/lab/native-map-folder.js','web/lab/raw-map-files.js']
            files.update(rows[-1]['native_kernel_sources'] + rows[-1]['browser_rust_sources'] + rows[-1]['browser_sources'])
        if identifier == 'bisect.ReplayNative':
            rows[-1]['browser_sources'].append('web/lab/native-recorded-replay-results.js')
        files.update([entry['source'], dispatch, *kernels, *browser_sources])
    actions = []
    for identifier, anchor, native, browser, stub in TUI_ACTIONS:
        assert anchor in (ROOT / TUI).read_text(encoding='utf-8')
        actions.append(dict(id='tui.' + identifier, native_dispatch=dict(path=TUI, anchor=anchor),
            native_behavior=native, browser_behavior=browser,
            classification='shared-limitation' if stub else 'browser-adaptation',
            runtime_verification='not-rerun-by-inventory', full_parity_claim=False))
    files.update(['crates/bisect-tui/src/widgets/command_palette.rs', 'crates/bisect-tui/src/widgets/glossary.rs',
                  'crates/bisect-tui/src/session.rs', 'crates/bisect-tui/src/plans.rs',
                  'web/lab/label-package-merge-files.js', 'crates/bisect-wasm/src/label_package_merge.rs',
                  'crates/bisect-wasm/src/label_reconcile.rs', 'crates/bisect-cli/src/analyze_label.rs',
                  'crates/bisect-cli/src/run_registry.rs','crates/bisect-wasm/src/label_report.rs'])
    files.update(SHARED_BROWSER_PACKAGE_SOURCES)
    result = dict(schema_version='bisect-practitioner-actions-v1', date='2026-10-23',
        scope='All native command/screen declarations and concrete TUI action groups. Source reconciliation only, not runtime/full-parity or official certification.',
        authority=[pin(name) for name in sorted(files)], rows=rows, tui_actions=actions,
        ranked_missing_workflows=RANKED_MISSING_WORKFLOWS,
        recorded_scope_notes=[dict(id='label.repository-application', contract='docs/specs/wasm-label-repository-installation.md'), dict(id='report.pdf-delivery', contract='docs/specs/wasm-report-pdf-delivery.md'), dict(id='label.native-summary-application', contract='docs/specs/wasm-native-summary-application.md'), dict(id='label.build-application', contract='docs/specs/wasm-label-build-application.md')],
        ranking_note='The prior bounded application shortlist has scoped delivery. Complete remaining source/map/host/recovery acceptance for successful medium/large rows, execute national rows and audit broader lifecycle gaps next. This is not an exhaustive completion claim; capability child contracts remain runtime acceptance authority.')
    OUTPUT.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')


def validate_source_boundaries(data):
    """Reject the source-role and operation substitutions found during reconciliation."""
    aggregate = next(row for row in data['rows'] if row['id'] == 'bisect.Aggregate')
    assert aggregate['browser_operations'][0] == 'aggregate-native-analysis', 'Native Aggregate must map to raw native aggregation, not only strict report aggregation'
    assert {'crates/bisect-wasm/src/native_aggregate.rs', 'crates/bisect-wasm/src/national.rs'} <= set(aggregate.get('browser_rust_sources', []))
    assert 'web/lab/native-aggregate-files.js' in aggregate['browser_sources']
    assert set(AGGREGATE_WORKSPACE_SOURCES) <= set(aggregate['browser_sources']), 'Missing current Aggregate workspace source mapping'
    assert set(SHARED_BROWSER_PACKAGE_SOURCES) <= {ref['path'] for ref in data['authority']}, 'Missing shared browser package authority'
    replay = next(row for row in data['rows'] if row['id'] == 'bisect.ReplayNative')
    assert 'web/lab/native-recorded-replay-results.js' in replay['browser_sources'], 'Missing source-bound replay export/handoff mapping'
    conflicts = next(row for row in data['rows'] if row['id'] == 'civic.Conflicts')
    assert conflicts['browser_operations'] == ['inspect-native-civic-conflicts']
    assert 'crates/bisect-analysis/src/civic.rs' in conflicts['native_kernel_sources']
    assert 'crates/bisect-wasm/src/native_civic_conflicts.rs' in conflicts['browser_rust_sources']
    for row in data['rows']:
        assert not any('/bisect-wasm/' in name for name in row['native_kernel_sources']), 'Browser adapter misclassified as native CLI kernel: ' + row['id']


def check():
    subprocess.run([sys.executable, str(ROOT / 'scripts/web/wasm_inventory.py'), '--check'], cwd=ROOT, check=True)
    data = json.loads(OUTPUT.read_text(encoding='utf-8'))
    entries = json.loads((ROOT / 'docs/specs/wasm-capability-inventory.json').read_text(encoding='utf-8'))['entries']
    assert data['schema_version'] == 'bisect-practitioner-actions-v1'
    assert {row['id'] for row in data['rows']} == {row['id'] for row in entries}
    assert len(data['rows']) == len(entries)
    pins = {ref['path'] for ref in data['authority']}
    for ref in data['authority']:
        path = ROOT / ref['path']
        assert sha(path) == ref['sha256'] and path.stat().st_size == ref['byte_length'], ref['path'] + ' drifted'
    operations = surface()
    validate_source_boundaries(data)
    assert data['ranked_missing_workflows'] == RANKED_MISSING_WORKFLOWS, 'Review the next source-backed practitioner actions'
    for item in data['ranked_missing_workflows']:
        ref = item['native_consumer']
        assert ref['path'] in pins and ref['anchor'] in code_mask((ROOT/ref['path']).read_text(encoding='utf-8'))
        assert item['browser_prepared_operation'] in operations
    for note in data['recorded_scope_notes']:
        assert (ROOT/note['contract']).is_file()

    for row in data['rows']:
        assert row['source_review_state'] == 'source-reconciled' and row['full_parity_claim'] is False
        assert row['runtime_verification'] == 'not-rerun-by-inventory'
        assert row['browser_operations'] and set(row['browser_operations']) <= operations, row['id']
        assert row['remaining'] and row['comparison_contract'] and row['assignee']
        for name in [row['native_declaration'], row['native_dispatch']['path'], *row['native_kernel_sources'], *row['browser_sources'], *row.get('browser_rust_sources',[])]:
            assert name in pins, 'Unpinned source: ' + name
        assert row['native_dispatch']['anchor'] in code_mask((ROOT / row['native_dispatch']['path']).read_text(encoding='utf-8'))
    assert len({row['id'] for row in data['tui_actions']}) == len(TUI_ACTIONS)
    for action in data['tui_actions']:
        assert action['native_dispatch']['anchor'] in (ROOT / action['native_dispatch']['path']).read_text(encoding='utf-8')
        assert action['full_parity_claim'] is False
    print(f"{len(data['rows'])} practitioner command/screen rows, {len(data['tui_actions'])} TUI action groups and {len(pins)} source pins valid; no runtime/full-parity claim")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--initialize', action='store_true')
    args = parser.parse_args()
    if args.initialize:
        initialize()
    check()
