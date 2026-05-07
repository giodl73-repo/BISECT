# Track D — VRA and Legal Implementation: Panel Revision Plan

**Generated from**: REVIEW_PANEL.md (2026-05-07)
**Track**: D-vra-legal
**Papers**: D.0 through D.5
**Module gate**: 3.0/4 — CONDITIONAL PASS

---

## Track-Level PP Items (Address First)

### PP1 — Post-Callais Acknowledgment (D.0, D.1, D.3)
**Scope**: D.0 §2.4, D.1 §2.2, D.3 §2.2 — one paragraph each
**Effort**: 1–2 hours per paper
**Deadline**: Before any law review submission

Add one paragraph to each paper's Background: (a) Louisiana v. Callais (2026) is the most recent Supreme Court Section 2 elaboration; (b) Callais addresses prong 3 disentanglement of racial from partisan bloc voting; (c) the prong 1 geographic compactness analysis in this paper is unaffected by Callais. Use D.5's existing Callais framing as the template.

---

### PP2 — Round 2 Reviews Required (D.1, D.3)
**Scope**: Both completed revisions but no documented Round 2 reviewer re-scoring
**Deadline**: Before submission of either paper

**For D.1**: Conduct Round 2 reviews with Pildes (primary) and Stephanopoulos. Focus specifically on whether the threshold framing is consistently maintained throughout the revised paper. The abstract language is adequate; reviewers must verify the Discussion does not revert to treating 42% as a legal test.

**For D.3**: Conduct Round 2 reviews with Pildes (primary) and Chen. Focus on whether the Shaw/Miller Pareto framework adequately addresses the constitutional permissibility question.

**Gate**: Neither paper should be submitted to any venue until R2 reviews are complete.

---

### PP3 — D.2 Full Panel Review Required
**Scope**: D.2 has zero completed reviews
**Deadline**: Before D.2 can be cited in D.4 or D.5

1. Downgrade D.2 `_panel.yaml` stage from `ready` to `draft`
2. Assign five reviewers — minimum: Pildes or Stephanopoulos (VRA law), Karypis (graph partitioning), Rodden or Chen (political science)
3. Key questions for reviewers: (a) Does the success-rate operationalization correctly represent Gingles prong 1? (b) Is the 4.5pp ceiling difference legally significant for borderline states? (c) Is the venue (Algorithmica vs. Political Analysis) appropriate for the VRA contribution?

---

### PP4 — Create Missing _panel.yaml Files (D.4, D.5)
**Scope**: Administrative — 30 minutes each

Create `research/D.4+legal-implementation/_panel.yaml`:
```yaml
paper: D.4+legal-implementation
venue: Harvard Law Review
stage: ready
round: 2
reviews_completed: 10
round2_avg_score: 4.0
gate_passed: true
pp_items:
  - id: PP-D4-1
    title: Arizona AIRC engagement
    status: open
  - id: PP-D4-2
    title: Geographic sorting footnote to main text
    status: open
  - id: PP-D4-3
    title: 69-district disaggregation by state partisan control
    status: open
  - id: PP-D4-4
    title: Originalist defense paragraph
    status: open
```

Create `research/D.5+gingles-bloc-voting-methodology/_panel.yaml`:
```yaml
paper: D.5+gingles-bloc-voting-methodology
venue: Yale Law Journal
stage: ready
round: 2
round1_avg_score: 2.8
round2_avg_score: 3.0
gate_passed: true
pp_items:
  - id: PP-D5-1
    title: Daubert reliability analysis
    status: open
  - id: PP-D5-2
    title: Case citations for alignment threshold calibration
    status: open
  - id: PP-D5-3
    title: Callais binding scope statement
    status: open
```

---

### PP5 — Partisan Neutrality Cross-Reference (D.3, D.4)

**D.3**: Add to Discussion section: "Partisan fairness analysis in [D.0, §5.4] demonstrates that edge-weighted plans produce efficiency gap of −0.02 (vs. −0.05 for enacted plans), confirming that the VRA-compactness optimization does not create partisan bias. This is consistent with the LULAC v. Perry requirement that VRA compliance not serve as pretext for partisan gerrymandering."

**D.4**: Move footnote 2 (geographic sorting / partisan tilt) to the Introduction's problem subsection (§1.2). Expand to full paragraph: (a) reflects voter geography, not algorithmic targeting; (b) exists equally in commission-drawn and court-ordered maps; (c) VRA addresses minority dilution, not partisan seat distributions. Cross-reference D.0 §5.4.

---

### PP6 — Reconcile the 137/68 Track Headline Claim (D.0, D.4)
1. Confirm source and methodology for the 137/68 national MM district count in D.0
2. In D.4's statute findings, add specific citation: "Based on [D.0, Table X / Supplementary Table Y]..."
3. Re-state under CVAP threshold or explicitly note VAP redistricting data definition
4. Add disaggregation by state partisan control (how many of the +69 surplus districts come from Republican-controlled vs. Democratic-controlled states)
5. Update MODULE.md if 137/68 figures change after CVAP reconciliation

---

## Per-Paper Revision Items

### D.0 Revision Items (for Yale LJ re-submission)

| ID | Priority | Action | Effort |
|---|---|---|---|
| D0-R1 | PP (track) | Add Callais paragraph to §2.4 Background | 1 hour |
| D0-R2 | P2 | Re-report 137/68 MM district comparison under CVAP definitions; reconcile with D.4 statute findings | 2 days |
| D0-R3 | P3 | Add footnote on post-Shelby enforcement significance for Section 5 study states | 2 hours |

**Gate for Yale LJ**: D0-R1 and D0-R2 must be complete. D0-R3 recommended.

---

### D.1 Revision Items

| ID | Priority | Action | Effort |
|---|---|---|---|
| D1-R1 | BLOCKING (submission gate) | Conduct Round 2 reviews (Pildes, Stephanopoulos) confirming threshold framing adequacy | 1–2 weeks |
| D1-R2 | P1 (verify in R2) | Verify throughout revised paper that no language treats 42% as self-executing legal standard | In R2 scope |
| D1-R3 | PP (track) | Add Callais paragraph to §2.2 Gingles Framework | 1 hour |
| D1-R4 | P2 | Add paragraph to policy implications section addressing post-Shelby differential significance for former Section 5 states | 4 hours |

**Gate for Election Law Journal**: D1-R1, D1-R2, D1-R3 must be complete.

---

### D.2 Revision Items

| ID | Priority | Action | Effort |
|---|---|---|---|
| D2-R1 | BLOCKING | Downgrade _panel.yaml stage to draft; assign five-reviewer panel; conduct R1 review cycle | 2–3 weeks |
| D2-R2 | P1 (in R1) | During R1 review: assess whether 4.5pp ceiling difference is legally significant for borderline states | In R1 scope |
| D2-R3 | P1 (in R1) | VRA law reviewer confirms compliance operationalization is legally defensible | In R1 scope |
| D2-R4 | P2 (strategic) | Decide venue: Algorithmica/INFORMS (computational) vs. Political Analysis/Election Law Journal (VRA contribution primary) | Pre-review decision |

**Gate**: No submission until R1 review complete.

---

### D.3 Revision Items

| ID | Priority | Action | Effort |
|---|---|---|---|
| D3-R1 | BLOCKING (submission gate) | Conduct Round 2 reviews (Pildes, Chen) | 1–2 weeks |
| D3-R2 | P1 (verify in R2) | Locate or correct the 85% compactness retention figure: identify specific table; if absent, correct MODULE.md | In R2 scope |
| D3-R3 | PP (track) | Add Callais paragraph to §2.2 VRA Compliance Background | 1 hour |
| D3-R4 | PP (track) | Add partisan neutrality cross-reference paragraph citing D.0 §5.4 | 2 hours |

**Gate for APSR**: D3-R1, D3-R2, D3-R3 must be complete.

---

### D.4 Revision Items

| ID | Priority | Action | Effort |
|---|---|---|---|
| D4-R1 | PP | Add Arizona State Legislature v. AIRC (2015) engagement to §2 (Constitutional Foundations) | 1 day |
| D4-R2 | PP | Move footnote 2 (geographic sorting) to §1.2 main text; expand to three-part response | 4 hours |
| D4-R3 | PP | Disaggregate the "+69 majority-minority districts" figure by state partisan control in §6.3 and statute findings | 1 day |
| D4-R4 | PP | Add 2–3 paragraph originalist defense to §2: Apportionment Act of 1842 as historical precedent | 4 hours |
| D4-R5 | PP (track) | Create _panel.yaml | 30 min |
| D4-R6 | PP (track) | Add D.0 citation + CVAP note + disaggregation in statute findings; coordinate with D4-R3 | Coordinated |

**Gate for HLR**: D4-R1 through D4-R6 all complete.

---

### D.5 Revision Items

| ID | Priority | Action | Effort |
|---|---|---|---|
| D5-R1 | PP | Add Daubert reliability analysis to §7: map each methodological choice to four Daubert factors | 1 day |
| D5-R2 | PP | List specific cases underlying 0.5 alignment threshold; add table (case name, circuit, year, Prong 1 outcome, alignment score) | 1–2 days |
| D5-R3 | PP | State Callais binding scope explicitly: SCOTUS (binding nationally) or sub-SCOTUS (persuasive only) | 2 hours |
| D5-R4 | P2 | Add paragraph to §3 (Prong 2) justifying "contested primaries" operationalization for cohesion | 3 hours |
| D5-R5 | PP (track) | Create _panel.yaml | 30 min |

**Gate for YLJ**: D5-R1 through D5-R3 and D5-R5 complete.

---

## Master Checklist

### Track-Level
- [ ] PP1-D0: Callais paragraph in D.0 §2.4
- [ ] PP1-D1: Callais paragraph in D.1 §2.2
- [ ] PP1-D3: Callais paragraph in D.3 §2.2
- [ ] PP2-D1: Round 2 reviews completed (Pildes, Stephanopoulos)
- [ ] PP2-D3: Round 2 reviews completed (Pildes, Chen)
- [ ] PP3-D2: D.2 downgraded to draft; full panel review conducted
- [ ] PP4-D4: _panel.yaml created for D.4
- [ ] PP4-D5: _panel.yaml created for D.5
- [ ] PP5-D3: Partisan neutrality cross-reference added to D.3 Discussion
- [ ] PP5-D4: Geographic sorting footnote moved to main text in D.4
- [ ] PP6: 137/68 headline reconciled; citation added; CVAP version stated

---

## Sequencing

**Phase 1 — Infrastructure and gating (Week 1)**:
1. PP4: Create D.4 and D.5 _panel.yaml files (30 min each)
2. PP3: Downgrade D.2 to draft and initiate panel review assignment
3. PP2: Initiate Round 2 review assignment for D.1 and D.3

**Phase 2 — Low-effort additions (Weeks 1–2, parallel with review turnaround)**:
4. PP1: Add Callais paragraphs to D.0, D.1, D.3
5. PP5: Add partisan cross-references to D.3 and D.4
6. D0-R3, D1-R4: Post-Shelby paragraphs
7. D5-R3: Callais binding scope statement
8. D5-R4: Prong 2 cohesion justification

**Phase 3 — Substantive revisions (Weeks 2–3)**:
9. D4-R1 (AIRC), D4-R2 (footnote move), D4-R4 (originalist defense)
10. D5-R1 (Daubert), D5-R2 (calibration cases table)
11. D0-R2 + PP6: CVAP reconciliation (coordinate D.0 and D.4 together)
12. D4-R3 + D4-R6: 69-district disaggregation and statute findings citation

**Phase 4 — Review-dependent items (Weeks 3–4)**:
13. D.1 and D.3 R2 review cycles (parallel)
14. D.2 R1 review cycle

**Track is submission-ready** on all five non-D.2 papers within approximately 3–4 weeks assuming reviews return on schedule.
