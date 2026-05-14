# CLI Weighted Cut Scoring Cleanup Close

## Outcome

Consolidated repeated weighted crossing-edge scoring in `bisect-cli` into a
local `weighted_edge_cut` helper.

## Evidence

- The helper sums weights for edges crossing a caller-provided left side.
- Repeated seed-selection scoring loops now call the helper.
- A direct unit test covers weighted crossing-edge summation.

## Boundary

Weighted cut scoring remains local because current reuse is within one CLI
module and weight validation/policy is tied to bisection-runner inputs. This is
not yet a `rgraph-core` API.
