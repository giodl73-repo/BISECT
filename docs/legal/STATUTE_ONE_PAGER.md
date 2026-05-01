# Reproducible Congressional Districting Act — One-Page Summary

**Status:** v0 draft, 2026-05-01.
**For:** Senate Judiciary / House Administration staff briefings.
**Full text:** `docs/legal/MODEL_FEDERAL_STATUTE.md`. **Policy memo:** `docs/legal/STATUTE_RATIONALE.md`.

---

## The problem

In *Rucho v. Common Cause* (2019), the Supreme Court held that partisan-gerrymandering claims are nonjusticiable in federal court — there is no "judicially manageable standard" for federal courts to apply. The Court explicitly invited Congress and the States to act through positive law. State courts and state commissions are working that path. **Federal legislation has not been seriously attempted as a single-issue bill.**

Existing federal districting law sets thin constraints: single-member districts (2 U.S.C. § 2c), one-person-one-vote (post-*Wesberry*), and VRA § 2 (52 U.S.C. § 10301). Within these, States have wide discretion — and that discretion is where partisan gerrymandering happens.

## The proposal

A federal statute requiring each State to draw its congressional districts using a deterministic, partisan-blind algorithm (recursive graph bisection on the census tract adjacency graph), and to publish reproducibility artifacts that allow any person to verify the map's compliance byte-identically.

The architecture mirrors **2 U.S.C. § 2a** (the Huntington-Hill apportionment statute): Congress prescribes the method; states execute; verification is mechanical.

## Why this works where prior reform bills have not

| Prior approach | Failure mode |
|---|---|
| "Compact districts" | Six accepted definitions; choice imports discretion |
| "Communities of interest" | Subjective; no manageable standard |
| Partisan-fairness metrics | Diagnostic, not procedural; doesn't tell line-drawer what to do |
| Independent commissions | Capture risk; no national standard |
| Bundled omnibus bills (HR 1) | Too many priorities; never reach 60 votes |

**This bill specifies a procedure with one output.** "Did the State run the prescribed algorithm on the prescribed inputs?" has a yes/no answer that any first-year computer-science student can verify in milliseconds. *Rucho*'s "no manageable standard" objection does not apply: the standard is mechanical execution, not substantive fairness.

## Constitutional posture

- **Authority**: Elections Clause, Art. I § 4 ("Congress may at any time by Law make or alter such Regulations" governing the manner of congressional elections). Settled by *Smiley v. Holm* (1932) and *Arizona v. Inter Tribal Council* (2013).
- **Precedent**: 2 U.S.C. § 2c (single-member districts) and *Wesberry*'s equal-population rule are existing federal mandates that constrain State line-drawing in the same way.
- **Federalism preserved**: States still execute. Census Bureau does not draw maps. Required local knowledge (VRA § 2 compliance, communities of interest) stays at the State level.
- **VRA § 2 carve-out (§ 106 of the bill)**: the algorithm baseline is race-blind; States may modify specific districts to comply with VRA § 2 with published *Gingles* justification; modifications are judicially reviewable on a deviation-from-baseline standard.
- **Disentanglement (§ 104(e))**: race and partisan considerations are addressed separately, satisfying *Louisiana v. Callais* (608 U.S. ___ 2026) p.36.

## What the federal contribution actually is

| Component | Federal | State |
|---|---|---|
| Algorithm specification | ✓ | |
| Parameter table (seed, tolerances) | ✓ (Director of Census + expert panel) | |
| Reference-implementation designation | ✓ | |
| Execution | | ✓ |
| VRA § 2 review and justification | | ✓ |
| Verification right | held by every citizen (private right of action) | |

The novel federal contribution is the **reproducibility regime**: the State publishes its `final_assignments.json` + `manifest.json`, any person reruns the algorithm, byte-identity is required (modulo published VRA § 2 deviations).

## Implementation status

The reference implementation already exists and is mature. This project (`redist`, github.com/<owner>/redist) has produced validated 50-state output for 2000 / 2010 / 2020, with per-plan provenance manifests, AEA-compliant replication packages, plan-comparison narrative reports, and bloc-voting analysis for VRA § 2.

**Algorithmic risk: zero.** The METIS graph-partitioning library has been in production scientific computing since 1997. The recursive bisection method is described in any graduate algorithms textbook. The `redist` reference implementation is open-source, MIT-licensed, and continuously tested.

## Implementation timeline

- **Year 1**: Coalition assembly (reform organizations + academic election-law scholars + civil-rights organizations + bipartisan good-government caucuses). Bill introduced in the Senate.
- **Year 2**: Hearings, mark-up, floor consideration. Realistic 60-vote path runs through small-state Senators framing this as a Huntington-Hill–style technical reform, not partisan reform.
- **Year 3**: Director of Census convenes expert panel, publishes parameter table, certifies reference implementation.
- **Year 4 (2030 census)**: First compliance cycle. States publish reproducibility artifacts within 180 days of decennial release.

## What this does NOT do

- Does **not** eliminate gerrymandering (radically narrows it).
- Does **not** address state-legislative districting (Congress's authority is congressional only).
- Does **not** address voter suppression, ID, ballot access (separate issues).
- Does **not** displace State independent commissions (commissions can run the algorithm).
- Does **not** require this specific reference implementation (any conforming implementation may be designated).
- Does **not** disturb VRA § 2 (preserves it explicitly; provides a baseline against which deviations are justified).

## The ask

Introduce a freestanding, single-issue Reproducible Congressional Districting Act. **Do not bundle** with HR 1–style omnibus voting-rights legislation. Single-issue framing is the difference between 45 votes and 60.

---

**Contact:** [project maintainer]. **Reference implementation:** github.com/<owner>/redist. **Licensure:** MIT. **Cost:** zero (the reference implementation already exists; the federal contribution is administrative — Census Bureau panel + parameter publication).
