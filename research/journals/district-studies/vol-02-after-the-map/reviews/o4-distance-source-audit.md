---
journal: District Studies
volume: 2
title: "After the Map"
status: source-chain-audit
updated: 2026-05-09
source: O.4 Constituent Distance
---

# O.4 Constituent Distance Source Audit

## Result

O.4 is the cleanest Vol. 2 anchor for public preview because the outcome is
directly computable from geography, routing, and office-location assumptions.
It still needs provenance notes before exact numbers are promoted.

## What The Source Supports

The current O.4 source supports a cautious claim:

> Constituent travel distance is a concrete outcome that can be computed from
> district geometry, road-network routing, and office-location assumptions.

It is stronger for District Studies than the turnout and polarization pieces
because it does not require the same behavioral causal identification.

## Evidence Found

| Field | Source detail | Publication effect |
|-------|---------------|--------------------|
| Office addresses | 435 House district office addresses collected from `house.gov` during March 2024 | Snapshot date visible; keep in source note |
| Geocoding | Census Bureau Geocoder API; 428 automatic, 7 manual Google Maps verification | Manual assignments need artifact/provenance before exact lock |
| Routing | OSRM 5.27 over January 2024 North America OpenStreetMap snapshot | Good provenance; still needs reproducibility artifact |
| Coverage | Contiguous 48 states; Alaska and Hawaii excluded | Must state scope |
| Counterfactual office location | Bisect districts use population-weighted centroid as office location | Important assumption; do not call measured actual travel time |
| Mechanism | Neck geometry/NWR used to explain distance | Useful, but exact NWR values need code/artifact check before lock |

## Language Allowed

- "O.4 is the strongest directly measured outcome candidate."
- "The source treats travel distance as a computable access measure."
- "The preview can explain the office-location and routing assumptions."

## Language Blocked

- "Bisect districts reduce constituent travel time" without saying this is a
  counterfactual comparison.
- Exact minute savings without routing/artifact provenance.
- Legal harm or court-facing language in District Studies.

## Recommendation

Use O.4 as the clean outcome anchor for Vol. 2. It should likely appear before
turnout and polarization, because the measurement chain is easier to explain
and less causally fragile.
