# Nationwide 2010 input inventory

This package freezes the local input custody state before the 2010 NRS
baseline is constructed.

- All 50 State PL 94-171 packages are present. Each package includes the
  fixed-width geography file, population segments 1 and 2, and the Census
  packing list.
- The geography files contain 11,071,790 block (`SUMLEV=750`) records.
- The cycle-correct apportionment configuration contains 435 congressional
  districts across 50 States.
- Delaware is the first live TIGER/RCTX pilot. Its 24,115-block context joins
  the 2010 fixed-width PL geography to `GEOID10` TIGER geometry exactly,
  contains 58,028 undirected land edges, needs no synthetic bridge, and passes
  the independent context verifier.
- The other 49 State TIGER inputs and RCTX files remain to be constructed.

Every retained PL and TIGER source is recorded with its relative path, byte
length, and SHA-256 digest in `inventory.json`. The inventory remains
`incomplete` until the missing TIGER inputs are acquired and all contexts are
built. It makes no district-generation or legal-validity claim.

Regenerate with:

```text
python scripts/research/inventory_national_2010.py
```
