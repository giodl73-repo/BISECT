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
- Say why the candidates are being tried. A figure should answer "what decision
  is this candidate set feeding?" before it asks the reader to compare scores.
- Include at least one numeric table, score, threshold, or ledger when the
  algorithm makes a quantitative decision.
- Show the split, not just the split label. A `1:13` ratio should be drawn as
  one part against thirteen parts; a `7:7` ratio should look visibly balanced.
- Prefer two-dimensional block miniatures for schematic spatial algorithms. If
  the algorithm acts on a state, region, county, grid, or district map, use a
  countable little array of blocks before falling back to blobs or
  one-dimensional bars.
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

## Split Depiction Rule

If an algorithm is about dividing seats, population, land area, graph units, or
district columns, the visual should depict the division directly:

| Concept | Weak visual | Better visual |
|---|---|---|
| `1:13` ratio | a box labeled `1:13` | a state thumbnail or bar split into one small part and thirteen parts |
| `7:7` ratio | a box labeled `7:7` | a state thumbnail or bar split into two equal halves |
| county split | text saying "county split" | county region visibly crossed by a boundary |
| selected frontier point | text saying "selected" | highlighted point on the frontier |

Labels can still name the split, but the geometry must carry the meaning first.

For spatial pages, include more than one geometry when it matters. A long block
array and a tall block array can make clear why the algorithm is trying several
candidate root splits instead of assuming one universal split shape.

## Candidate Purpose Rule

When a visual shows multiple candidate splits, seeds, columns, moves, or
frontier points, it should state the downstream decision they feed:

| Candidate set | Downstream decision |
|---|---|
| GeoSection ratios | root allocation of district counts before recursion |
| AreaSection ratios | feasible root allocation under population and area windows |
| seed stream | which deterministic candidate plan is selected |
| branch-and-price columns | which district columns cover all units once |
| local-search moves | which validity-preserving move becomes the descendant plan |

If the reader cannot tell why the candidates exist, the figure is not finished.
