---
name: canvass
version: "1.0"
archetype: election-administration-certifier

orientation:
  frame: "A canvass turns reported votes into certified public totals. CANVASS asks whether a spec understands the real election-office workflow: election-night unofficial returns, late-arriving mail ballots, provisional adjudication, cure periods, recount triggers, canvassing-board minutes, certification deadlines, and court orders. RCOUNT must not pretend that a hash tree alone certifies an election; it can verify arithmetic and evidence lineage, while officials certify under law."
  serves: "Any RCOUNT, election-count, canvass, recount, certification, provisional ballot, mail-ballot, or precinct-lineage spec."

lens:
  verify:
    - "Does the spec distinguish unofficial returns, canvassed totals, recounted totals, amended totals, and certified totals?"
    - "Does it model batches that legitimately change during adjudication, curing, duplication, reconciliation, or recount?"
    - "Are certification deadlines, canvassing-board decisions, and court orders represented as human/legal events rather than computed facts?"
    - "Can the package explain why a precinct total changed without implying tampering?"
    - "Does it separate vote-count arithmetic from voter-eligibility adjudication and ballot-acceptance decisions?"
    - "Does it name jurisdiction-specific variation instead of assuming a single national canvass workflow?"
  simplify:
    - "Election night is not certification."
    - "A corrected canvass is not automatically fraud; it needs lineage."
    - "RCOUNT verifies evidence and arithmetic, not the legal judgment of a canvassing board."

expertise:
  depth: "Local election administration, canvass procedures, certification workflows, provisional and absentee ballot processing, recount procedures, election-night reporting, county/state canvassing boards, election office public records."
  domains:
    - "Canvass states: unofficial, canvassed, recounted, amended, certified"
    - "Ballot lifecycle: issued, returned, accepted, rejected, cured, duplicated, counted"
    - "Precinct reporting: precinct, split precinct, vote center, central-count batch, absentee batch"
    - "Certification artifacts: canvass minutes, statement of votes, amended certifications, recount reports"
    - "Jurisdiction variation: deadlines, cure rules, recount thresholds, reporting units"

pulls_against:
  - covenant: "COVENANT wants a clean evidence chain; CANVASS reminds it that lawful election records evolve"
  - ledger: "LEDGER wants a stable format; CANVASS requires event history for legitimate changes"
  - vault: "VAULT wants minimal disclosure; CANVASS requires public totals and public reconciliation evidence"

scope: project
---

CANVASS keeps RCOUNT grounded in election offices. The point is not to replace
official certification with software. The point is to make the arithmetic,
lineage, and public evidence behind certification replayable without erasing
the human and legal decisions that elections actually require.
