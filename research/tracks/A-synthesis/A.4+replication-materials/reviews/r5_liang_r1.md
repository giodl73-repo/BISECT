> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Round 1 Review — Percy Liang
**R1 Score: 2.9/4.0**

## Summary Assessment

Reproducibility packages should be evaluated against a simple standard: can an independent researcher follow the steps and reproduce the results? I read this package as that researcher. The procedure is mostly followable, but I identified one CLI error that will block reproduction, several ambiguities that would require the reader to consult the codebase rather than the paper, and a significant gap in the SHA verification steps as documented.

## Step-by-Step Procedure: What Fails

**Step 3 — `redist fetch --verify-sha256`.** This flag does not exist. The actual flag is `--verify-downloads` (confirmed in the CLI source, `FetchArgs` struct, line 2499). A researcher following Step 3 exactly as written will receive an error and be unable to proceed. This is a blocking error and must be corrected. It is the single most important fix needed in this document.

**Step 4 — Output path.** The command `redist run --year 2020 --version official_2020` is stated to write outputs to `outputs/official_2020/2020/{state}/`. This double-year path (`official_2020/2020/`) looks unusual. If correct, it should be confirmed; if the actual path is `outputs/official_2020/{state}/` the paper must be corrected.

**Step 6 — Placeholder hashes.** All four SHA-256 values in Table 2 are zeros. A researcher following the paper cannot verify their run against published values until these are populated. The paper correctly states these are placeholders, but this means the package is currently incomplete as a verification tool. The paper should specify how a researcher will be notified when real hashes are published — is there a CHANGES.md? A release tag? A note in the repository?

## Placeholder Strategy: Acceptable but Requires a Plan

The use of placeholder SHA values throughout (Tables 1, 2, 3, and 4) is noted as intentional — "to be confirmed after the first clean run." This is an acceptable approach for a pre-publication package. However, for the package to actually fulfill its AEA obligation, there must be a mechanism to replace all placeholders before the portfolio is formally published. The paper should document this process: who runs the clean reference run, on which hardware, when, and how the hashes are published in the repository.

The Vermont walkthrough fixture (`pin.sh`) is a good model for this process. The paper should state explicitly that the same pin-and-publish workflow will be applied to the full 50-state runs before publication.

## SHA Verification: Independence

The paper states that `redist label-verify` "recomputes the SHA-256 of every config, input file, and output file." This is verified by the CLI source. However, for true independent reproducibility, the verification should not require the `redist` tool itself — an independent researcher should be able to compute SHA-256 values using any standard tool (e.g., `sha256sum` on Linux, `certutil -hashfile` on Windows, Python's `hashlib`) and compare against published values. The paper should provide the formula for combining individual file hashes into the build hash (see Duchin's review for the binary-vs-ASCII ambiguity), so that independent verification is possible without the `redist` binary.

## Software Stack: Python Version

Table 1 lists "Python 3.13+" as a requirement for dashboard and research. Python 3.13 was released in October 2024 and is not yet the default on most Linux distributions or macOS. Researchers on standard university cluster environments may have Python 3.10 or 3.11. The paper should document whether Python 3.13 is a hard requirement or a recommendation, and what functionality is unavailable on earlier versions.

## Test Count Consistency

The paper reports ~1,150 total tests. The CLAUDE.md project documentation states "1187 total workspace tests pass" as of the most recent change note. These are approximately consistent. The discrepancy (37 tests) may reflect tests added since the paper was written, which is fine, but the paper should note that test counts may vary by checkout date and that the reported numbers correspond to a specific repository version (`v0.2.0`).

## Recommendation

Major revisions needed. The `--verify-sha256` CLI error is blocking and must be fixed. The placeholder publication workflow needs to be documented. The SHA hash computation formula should be specified in enough detail for tool-independent verification. These are all achievable in a revision pass; the package's overall design is sound.
