# Native external split certificate

`certified-recursive-bisection-external-split-certificate-v1` is the native Rust contract for a split whose compact decision proofs have been checked by the concrete VeriPB kernel. The implementation is in `web/proof-checker/src/native_split.rs`, shared by the native executable and WASM artifact. It directly calls the existing `bisect-ilp` compiler and the actual checker; submitted checker receipts never substitute for proof checking.

This is a separate schema from the existing bounded exhaustive split certificate. Existing exhaustive transcripts and recursive tree validators are unchanged. The native proof page exposes explicit issuance and certificate download after a fresh complete run or saved-proof reverification. Issuance itself repeats compilation and every native proof check. The separate [external recursive tree contract](native-external-tree.md) now verifies proof-carrying node collections and native induced-child lineage; its UI and native plan delivery remain unfinished.

## Issuance and verification

The checker JSON boundary accepts:

```json
{
  "operation": "adopt-native-external-split",
  "instance_json": "<exact native split instance JSON>",
  "discovery_json": "<exact native discovery JSON>",
  "proofs": {
    "01-population": "<PBP proof text>",
    "02-boundary": "<PBP proof text>",
    "03-canonical": "<PBP proof text>"
  },
  "expected_certificate": null
}
```

The proof map must contain exactly every required native decision, including both `02-low-boundary-right-N` and `02-high-boundary-right-N` branches when required. The native compiler validates the incumbent's hash, objective, assignment and connectivity. Each request ID and OPB hash is checked before VeriPB verifies the actual proof bytes with UNSAT required. A missing decision, extra decision, SAT proof, incomplete proof, unchecked assertion or invalid proof prevents issuance. The operation uses no exhaustive oracle and does not generate proofs.

Successful issuance returns the exact native model and node/parent identities, source hashes, compiler source hash, optimal assignment/objective and checked decision evidence. `proof_kind` is `veripb-compact-decisions`. Every decision records its native request ID, OPB hash, exact pretty-serialized `request_json`, PBP hash and concrete checker report. The certificate ID is `sha256:` plus SHA-256 of canonical JSON for the entire certificate excluding `certificate_id`. Canonical object keys are sorted by the native JSON value representation.

Objective u64 values serialize as decimal strings. Native request JSON is retained as text, preserving i64 population branches beyond JavaScript's safe integer range. Native Rust still computes with the original integer types; native and WASM certificate values and IDs match.

Supplying an `expected_certificate` repeats native compilation and every proof check, then compares the entire freshly issued certificate with the supplied value. Recomputed public hashes cannot authorize an invalid proof or changed certificate. Proofs and exact input sources must accompany verification; a standalone certificate JSON is insufficient.

The scope is the unique canonical optimum for the supplied split model under the native compact encoding. Correct recursive node/parent assembly, global plan optimality, source authenticity and legal compliance require separate verification. A child split certificate's parent ID records its declared link; complete tree verification must check the parent and induced child instance.

## Limits and tests

Source JSON is limited to 3 MiB each, the graph to 100000 units/200000 edges, compiler estimated working set to 128 MiB, model package to 25 MiB, each proof to 64 MiB and total proof text to 128 MiB. The existing checker request and individual formula limits still apply. No input filenames or command templates are executed.

Build and test:

```powershell
python scripts/web/build_proof_checker_wasm.py
python scripts/web/prepare_native_external_fixture.py
node scripts/web/test_native_external_certificate.mjs
```

The pipeline tests must first supply their three/five-decision proof fixtures. Native certificate tests compare complete native/WASM outputs and repeat expected-certificate verification, rejecting corrupted proofs, missing decisions, forged assignments and the checked canonical SAT counterexample. A separate two-unit fixture generates five actual proofs with population `18014398509481985`, verifies exact objective strings and a branch population above `2^53`, and repeats certificate verification without rounding. Evidence is in `target/native-external-certificate/evidence.json`. These tests establish the contract on the supplied fixtures, not statewide proof capacity.
