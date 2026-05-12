# Algorithm Atlas Visual Patterns

These notes capture the pattern from the upgraded GeoSection figure so other
atlas pages can improve in the same direction.

## Gold-Standard Figure Shape

A strong algorithm visual should usually show three things in one reading path:

1. The tempting failure or naive interpretation.
2. The algorithm's actual decision rule with numbers.
3. The downstream consequence for the plan, tree, certificate, or output.

For GeoSection, that became:

```text
raw tiny split -> normalized ratio table/chart -> chosen root split and recursion
```

## Design Rules

- Prefer one large teaching figure over several tiny decorative figures.
- Put the failure mode and the selected behavior in the same visual frame.
- Include at least one numeric table, score, threshold, or ledger when the
  algorithm makes a quantitative decision.
- Label whether higher or lower scores win.
- Show the artifact consequence: selected split, emitted report, audit package,
  rejected move, resample genealogy, or frontier selection.
- End with a one-sentence reading rule that tells the reader how to interpret
  the whole figure.

## Reusable Layout

```text
title and one-line thesis

[1. failure/naive view] -> [2. decision rule] -> [3. selected consequence]

reading rule footer
```

This pattern should be applied next to AreaSection, Seed Search Modes,
County-Sticky Weights, and U.20 because those pages benefit most from seeing
the algorithm's decision rule and its artifact consequence in the same figure.

## Applied Examples

- GeoSection: raw tiny split -> normalized ratio score -> chosen recursive root.
- AreaSection: population-only tiny footprint -> population/area feasibility
  window -> feasible ratios searched and selected.

When a page has two constraints, make both visible in the decision panel. Do not
hide the secondary constraint in prose; show the pass/fail window directly.
