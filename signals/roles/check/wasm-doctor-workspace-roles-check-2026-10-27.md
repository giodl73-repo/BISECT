---
skill: roles-check
topic: wasm-doctor-workspace
date: 2026-10-27
roles_used: 6
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

Standard single-agent application of installed lenses, not independent signatories.
Artifacts: native Doctor/registry/policy/manifest consumers, new Rust operation,
closed project recipe, metadata-only collectors, guided UI, native/browser and
boundary tests, and staged binaries. BENCHMARK: evidence; COVENANT: provenance;
LEDGER: exact data; VAULT: authority; SURVEY: usability; TRENCH: failure prevention.

| ID | Lens | Finding | Severity | Resolution or condition |
|---|---|---|---|---|
| B1 | BENCHMARK | Comments do not capture actual loader/path behavior. | P2 | Forty-three actual CLI stdout/status cases per profile. |
| B2 | BENCHMARK | A passing browser could bind a changing package manifest. | P2 | Start/end manifest equality and final exact stage binding; early observation excluded. |
| B3 | BENCHMARK | Registration/fixture counts could imply full parity. | P3 | Whole-builder and broader matrix remain open. |
| C1 | COVENANT | Imported snapshots could appear live/authenticated. | P2 | Explicit supplied-data scope, inert Open and fresh explicit Run/export. |
| C2 | COVENANT | Native stderr could be fabricated from browser IO. | P2 | Native stderr retained independently; visible typed source attempts. |
| C3 | COVENANT | New source pins might replace historical runtime identities. | P3 | Only current authority/mapping pins refreshed; old runtime receipts unchanged. |
| L1 | LEDGER | Counts beyond JS exact range or wasm32 width could truncate. | P2 | Existing generic typed manifest with u64; exact string/native wide-count case. |
| L2 | LEDGER | Valid scalar JSON must stop fallback before unknown lookup. | P2 | Rust Value parser preserves native stop; early-failure tests. |
| L3 | LEDGER | Case/alias dropdown normalization can discard original args. | P3 | Imported literals added as exact options; source-edit/request comparisons pass. |
| V1 | VAULT | Templates/display prefixes might grant outside-root reads. | P2 | Pure snapshot queries only; native absolute-path success explicitly unsupported. |
| V2 | VAULT | Metadata review could read artifact bodies or publish incomplete absence. | P2 | Body-read counter zero; enumeration errors/cancel refuse atomically. |
| V3 | VAULT | A forged imported path tree could describe impossible ancestry. | P3 | Closed records, directory ancestry and case-alias guards tested at project/kernel boundaries. |
| S1 | SURVEY | Native TIGER PASS can mean an unrelated entry. | P2 | Native wording retained with explicit presence-only boundary and junk/subdir cases. |
| S2 | SURVEY | Source read/parse refusal could be hidden behind loaded file labels. | P2 | Visible separate registry/tolerance/manifest summary plus full JSON attempts. |
| S3 | SURVEY | Advanced controls could overwhelm ordinary preflight. | P3 | Opt-in collapsed native mode; original policy/RCTX flow preserved. |
| T1 | TRENCH | Cancelled A can release B or overwrite Open. | P2 | Owner token/revision checks and real browser late/reentrant journey. |
| T2 | TRENCH | Refresh during input events can restore old mode/prefix values. | P2 | Capture user values/file before invalidation; mode/prefix/draft browser checks. |
| T3 | TRENCH | New Rust codegen could break C backend. | P3 | New C module passes 52 native assignments/four failures; broader execution remains open. |

Roles: 6. P1: 0; P2: 12; P3: 6. APPROVED-WITH-CONDITIONS for bounded
Windows/64-bit selected-snapshot Doctor adaptation. Top finding: registry, tolerance
and manifest loaders have different precedence/error semantics. BENCHMARK/LEDGER
agree on actual source behavior; VAULT/COVENANT agree on snapshot authority limits.

Amendments: separate loaders and exact CLI cases; root/path and source-attempt
boundaries; guarded opt-in UI with exact imported arguments and stable staged
identity. Existing decoder/shared-owner/reentrant classes apply; no distinct new
pitfall class is claimed. Wider paths/hosts/capacity, wide plan metadata, whole
builder, clean-checkout and formal acceptance remain open.
