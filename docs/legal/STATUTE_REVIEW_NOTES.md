# Statute Review Notes: v0.2 Decisions And Open Questions

**Status:** 2026-07-10
**Bill:** `MODEL_FEDERAL_STATUTE.md` v0.2
**Technical standard:** NRS v0.1

## Decisions Made

1. **Benchmark, not mandatory final map.** The State must publish and explain;
   it need not enact the benchmark.
2. **Blocks, not tracts.** Tract runs remain research fixtures.
3. **Standard bisection, not ApportionRegions.** Prime-factor structure remains
   a comparator.
4. **Geographic weights, not county-sticky weights.**
5. **One manifest-derived seed, not convergence T=600 or an agency-selected
   seed.**
6. **Equality as nearly exact as practicable, not a statutory 0.5% safe
   harbor.**
7. **No universal modification cap.** Authority, alternatives, concentration,
   and effects govern.
8. **COI criteria permitted with precommitment and evidence.**
9. **VRA modifications mandatory; benchmark never a defense.**
10. **Congress owns assignment-affecting changes; agencies own technical
    custody only.**
11. **Technical Schedules A and B are incorporated.** The bill does not become
    effective until block-level implementation, conformance corpus, reference
    service, reviewer program, and registry are certified.
12. **EAC is the lead agency and NIST is the technical custodian.**
13. **A new $250 million decennial program funds States, communities,
    reviewers, the reference service, language access, and conformance.**

## Resolved v0.1 Conflicts

| v0.1 conflict | v0.2 disposition |
|---|---|
| Binary statute vs prime-factor paper | Standard bisection is canonical; B.02 is comparator advocacy |
| Geographic statute vs county quickstart | Geographic only |
| Director-selected seed vs content-derived seed | Complete manifest-derived formula in statute |
| Tract unit vs population equality | Blocks normative |
| Per-level tolerance vs final cap | Legal equality rule; report optimizer tolerance separately |
| Mandatory algorithm output vs local legal duties | Benchmark evidence plus governed final plan |
| Fixed 1.5% modification cap | Removed |
| COI prohibited vs community legitimacy | Precommitted COI authority permitted |
| Byte-identical binary implication | Functional canonical assignment equivalence |

## Open Constitutional Questions

### Elections Clause scope

Congress plainly may regulate congressional elections. The unresolved question
is how far it may direct State officers to execute a federal disclosure process.
The v0.2 bill reduces risk by allowing federal benchmark generation while
leaving final map adoption to State law.

### Anti-commandeering

The primary argument is that these are Elections Clause election rules. The
bill also provides a federal service, grants, and a conditional-funding fallback
so the reform does not depend on one theory.

### Preemption remedy

Making a procedurally deficient map ineffective is stronger than ordering
publication alone. Legislative counsel should test whether a cure-and-
reconsideration remedy is sufficiently related to the Elections Clause
procedure.

### Standing

Bare procedural injury may not satisfy Article III. The cause of action should
track concrete voter, candidate, organizational, and election-administration
injury rather than create citizen standing by declaration.

## Open VRA Questions

The project must not freeze a changing Section 2 doctrine into source code.
The statute therefore refers to controlling law rather than codifying a
project-specific threshold.

Public VRA reasons improve accountability but can create a record relevant to
racial-predominance review. Options for sensitive evidence include:

- public legal conclusions with protected expert appendices;
- in-camera treatment when ordered by a court; and
- a State-generated VRA plan prepared before comparison with the benchmark.

The nonnegotiable rule is that benchmark non-modification creates no safe
harbor.

## Open Community Questions

- What evidence defines a community?
- Who may submit it?
- When is the criterion fixed?
- How are conflicting communities ranked?
- What disparate-impact tests apply?
- How does a commission explain rejection?

The bill requires a public definition, precommitment, alternatives, and reasons
but does not impose one national substantive community hierarchy.

## Open Technical Questions

- Native block adjacency at national scale.
- Water/island/enclave profile.
- Manifest-derived seed implementation.
- Reference-engine source and build profile.
- Canonical schema registry and conformance CLI.
- Data publication and long-term custody costs.
- Defect classification: implementation bug versus profile change.

## Political Tradeoffs

The v0.2 proposal is less rhetorically simple than "the algorithm draws the
map," but more legally and democratically defensible. It can complement current
commission proposals rather than displace them.

The coalition must decide whether the bill should:

1. require only benchmark publication;
2. require benchmark publication plus final-plan decision records (current);
3. create federal minimum criteria for final maps; or
4. require independent commissions as well.

These choices should be explicit committee decisions, not hidden in software
defaults.

## Review Gate

Before external legislative use:

- constitutional counsel reviews all four implementation postures;
- VRA plaintiffs' and defense counsel review Section 106;
- State administrators test timelines and costs;
- community organizations test the public process;
- independent engineers reproduce the benchmark package; and
- legislative counsel converts the model text into codified form.
