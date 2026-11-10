# Ceremony CLI and browser contract

Later delivery: [Start from original sources](wasm-ceremony-source-start.md) now
adds the guided source/profile form and exact native u64 review intervals.
The older records below retain their original scope and source identities.

Date: 2026-10-27. Status: bounded actual CLI/current-source browser comparisons
pass; final-package, broader capacity/host and formal parent acceptance remain open.

Subsequent checkpoint: the [complete local C preview](wasm-cffi-full-preview-checkpoint.md)
also passes both registered profiles and the final-package ceremony checks.
The earlier source-overlay evidence remains bound to its own bytes. Broader
capacities/hosts, original-profile preparation and formal parent acceptance remain open.

The desktop `ceremony verify` command first checks the transcript and released
prefix. For a completed transcript without `--tree`, it then refuses with
`completed ceremony verification requires --tree for final binding`. Supplying
the complete tree permits final binding. Open and halted transcripts can be
verified as prefixes without a tree.

The browser previously exposed the shared prefix checker through
`verify-ceremony`, accepting a completed transcript without the complete tree.
Its scope label was explicit, but this was not the CLI's failure contract.
Shared ToolRequest/native-adapter comparisons did not detect that orchestration
difference. Actual CLI reproduction and the failing-before regression are retained
under `target/ceremony-native-parity-audit`.

`verify-ceremony` now follows the desktop rule and failure order. The useful
browser adaptation remains available separately as `inspect-ceremony-prefix`:
it checks released evidence, including a completed transcript's released prefix,
without claiming final tree binding. The operation selector names the distinction,
hides irrelevant tree controls for inspection and explains completed verification's
tree requirement. Imported projects retain their operation as data; Open never
executes either operation. Older completed-prefix projects using Verify must
explicitly select inspection or supply the tree before their next Run.

The actual CLI contract test creates a synthetic eight-unit, four-district tree,
then runs the native CLI to Start, Announce to completion, Halt an open transcript
and Verify open/halted/completed transcripts with and without a tree. Browser
Start/Announce/Halt transcripts match every JSON value and array order. Native
pretty JSON and browser compact JSON are retained separately; file-byte equality
is not claimed. The completed-without-tree error matches exactly. A separate
browser prefix inspection succeeds with its limited scope.

The actual workbench/worker test exercises six explicit Runs: completed-tree
refusal, prefix inspection, full-tree verification, Start with supplied config,
Announce and Halt. It compares transcript values with actual native CLI files
and verifies inert prefix Save/Open. The project-handler suite also covers the
new operation, hidden controls, tree-free request and saved operation. Its
minimal DOM now implements `createTextNode`, allowing the current report-source
control installer to load without replacing it with a stub.

Current local evidence:

- `target/ceremony-native-contract/current-cli.json` identifies the actual CLI run.
- `target/ceremony-native-contract/browser-CJxx9a/evidence.json` records the six
  current-source UI/worker Runs.
- `target/ceremony-native-contract/shared-adapter-final.log` records 29 exact
  shared-adapter operations and 34 matching refusals, including exact i64 checks.
- `target/ceremony-native-contract/project-fixed.log` records project-handler
  regression checks.

The CLI and browser checkers are registered in both build variants' legacy gates.
The C preview builder also checks its final package, with source identity enforced
by `--require-package`. These registrations do not assert that the full updated
builder has passed. Its preceding attempt stopped at stale Fetch source identities;
reviewed C-recovery-only changes have a separate source-history entry and 42 fresh
native planner/FLETCH comparisons. Earlier runtime receipts remain unchanged.

Remaining gaps include guided preparation from original rule-profile bytes,
immutable filesystem publication/installation semantics, wider tree/data/host
coverage and formal acceptance bundles. Witness IDs, timestamps and rule-profile
identities are claims, not externally authenticated evidence. These checks do not
publish anything, certify an election, establish legal validity or complete
WP-01/WP-07/full desktop parity.
