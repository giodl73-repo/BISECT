# R2 Review — Percy Liang
**Paper**: U.1 ConvergenceSweep: T=600 Statutory Seed Formula
**Round**: 2
**Score**: 3.5/4
**Verdict**: Accept with minor revisions

## Assessment of P1 Items from R1

### METIS determinism qualifier (NTHREADS=1) — RESOLVED

Proposition 1 (Determinism) is now correctly qualified: its statement specifies "provided METIS is invoked with METIS_OPTION_NTHREADS=1 (single-threaded mode), regardless of hardware or operating system." The phrase "regardless of parallelisation strategy" that I flagged as too strong has been removed. The proof adds the necessary qualifier: "METIS is deterministic given s when run single-threaded: the multilevel coarsening and refinement sequence is fully specified by the seed and graph structure. METIS's OpenMP-parallel variants use a non-deterministic work-stealing scheduler; the statutory build therefore sets METIS_OPTION_NTHREADS=1 at initialisation, ensuring thread-count-independent determinism." A subsequent Remark reinforces the point with practical guidance for independent verifiers: "independent verifiers must use the same setting to reproduce the certified partition." Section 5.2 (Federal Statute Invocation) adds a "METIS thread-count requirement" paragraph stating that the redist binary automatically sets METIS_OPTION_NTHREADS=1 in statutory mode (--search convergence), and that a statutory certificate produced by a multi-threaded METIS build is not reproducible across machines and shall not be accepted as a certified alternative for litigation purposes. The qualifier is stated in three places (proposition, proof, implementation section), which is appropriate for a reproducibility claim with legal weight. This resolves my P1 concern fully.

### Cargo.lock accessibility (P2) — NOT RESOLVED

My P2 request for a URL to the specific repository commit and Cargo.lock used for the 50-state B.7 sweep is not addressed in the revised text. Section 5.1 still states "The binary ships with a vendored METIS library pinned to a specific source hash in Cargo.lock" and "The SHA-256 computation uses the Rust sha2 crate (version pinned in Cargo.lock), which is FIPS 180-4 compliant" — but no URL, no commit SHA, and no statement that the Cargo.lock will be published upon acceptance is present. For the "certifiably optimal" claim to be independently verifiable, this gap remains. A sentence of the form "The exact Cargo.lock used for the 50-state B.7 sweep is available at [URL] at commit [SHA], or will be published upon acceptance" is still needed. I am prepared to accept the paper with this as a condition to be satisfied before final publication, since it is a P2 disclosure item rather than a mathematical error — but it must appear in the final text.

### Version string binding to census cycle — RESOLVED

Section 4.3 now contains an explicit "Version string is bound to the census cycle, not the statute version" paragraph. The text states that the version string in effect on the date of the Census Bureau's public release of the redistricting dataset identifier governs the entire redistricting cycle — including any subsequent court-ordered redistricting — and that a subsequent DIA amendment adopting "DIA_SEED_V2" applies only to the next decennial census release. The example given ("A 2032 challenge to a 2021 V1 map cannot argue that DIA_SEED_V2 should have applied: the version string is locked at the census release date, not at the date of the challenge") is exactly the litigation scenario I was concerned about. Section 5.4 adds a statutory clause (D) "Version binding" that encodes the same rule in proposed statutory language: "The version string used in the seed formula specified in clause (A) shall be the version in effect on the date of the Census Bureau's first public release of the redistricting dataset identifier for the applicable decennial census, and shall not be changed for any redistricting conducted pursuant to that census, including court-ordered redistricting." Both the algorithmic-section prose and the statutory-text clause are present. This resolves my R1 concern.

## Remaining Concerns

### Table 5 runtime labelling (P2) — PARTIALLY ADDRESSED

My P2 concern about whether Table 5 reports measured or estimated runtimes is partially addressed. The caption now specifies the hardware ("AMD Ryzen 9, 32 GB RAM") and describes the values as "estimated wall-clock times... based on observed METIS call times." The j_stop column is present. However, the distinction between a measured T(M) per-seed time and a derived total — which I asked for explicitly — is still not present. The caption says "estimated... based on observed METIS call times" but does not show T(M) as a separate measured quantity with the total labelled as "derived: j_stop * T(M)." This is a P2 item and does not block acceptance, but a footnote noting the per-seed measurement baseline would complete the reproducibility chain.

### ConvergenceSweep with zero-seed-variance structures (P3) — NOT ADDRESSED

My P3 item about how ConvergenceSweep behaves when --structure prime-factor (ApportionRegions) is used — which T.4 established has zero seed variance, so the sweep would terminate at T=1 — is not addressed in the revised text. Section 5.2 describes the statutory invocation with --structure prime-factor and --search convergence without noting that this combination would produce trivial sweep behavior for zero-variance structures. This is a minor point and does not affect the paper's statistical claims, but practitioners who invoke the statutory command and observe immediate termination could incorrectly conclude the sweep failed.

## Recommendation

Accept with minor revisions. Both P1 items that are resolvable by text changes are fully resolved: the NTHREADS=1 qualifier is added in three places with appropriate legal framing, and the version string binding clause is present in both the algorithmic section and the proposed statute. The Cargo.lock URL (P2) is the one outstanding item that must be resolved before final publication — this is a disclosure obligation that comes with the paper's reproducibility claim, not a mathematical issue. The Table 5 labelling and zero-variance behavior (P3) are minor refinements. The paper's reproducibility infrastructure — NTHREADS=1, Cargo.lock pinning, sha2 crate version, FIPS 180-4 compliance, tamper-evident JSONL log — is now correctly specified and the determinism claim is no longer vulnerable to the verifier-obtains-different-plan attack I raised at R1.
