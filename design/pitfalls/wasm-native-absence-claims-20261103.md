# Missing native sources are explicit claims

Status: MITIGATED in the tested source candidate; amended release acceptance pending.

An unselected file is not evidence that it is absent. Conversely, making every
null-byte source invalid hides native fallback and required-source failure
behavior from browser users. The panel now requires a recorded filename and an
explicit action to create `{name, missing: true}`. The native transport retains
`bytes_base64: null`. Neither representation grants disk access, proves absence
on the user's computer, authenticates a publisher or activates a saved run.

Only COI, partisan, demographics, economic and housing accept this representation.
Exact-field validation rejects combined missing/file records, unsafe names and
unknown fields. Graph/geometry companions still require real selected bytes.
Editing a claim removes it and retires current outputs; selecting bytes replaces
it. Save/Open, archives and typed session restore retain data without execution.

`scripts/web/test_fresh_native_absence_sources.mjs` retains 15 positive cases and
29 malformed controls. `test_fresh_native_absence_browser.mjs` uses actual CLI
and both browser WASMs: COI/economic/housing fallback plans match literally;
required demographics and partisan sources fail on Run and leave no current
plan export. Native error wording is not projected into a false equality claim.
Both profiles cover edit/replacement and inert restore. The pre-existing
selected-file browser cases also pass. Physical context, exact integer balance
and every district's connectivity are independently checked for all 14 plans.

Retain the original failed test assumptions and fixtures. The original full
standard/C/browser/package/proof sequence and all original parent acceptance
requirements remain open. Small 16-unit AL fixtures do not establish full-state
capacity, source authenticity or legal sufficiency.
