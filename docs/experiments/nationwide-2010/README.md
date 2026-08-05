# Nationwide 2010 input inventory

This package freezes the local input custody state before the 2010 NRS
baseline is constructed.

- All 50 State PL 94-171 packages are present. Each package includes the
  fixed-width geography file, population segments 1 and 2, and the Census
  packing list.
- The geography files contain 11,071,790 block (`SUMLEV=750`) records.
- The cycle-correct apportionment configuration contains 435 congressional
  districts across 50 States.
- No 2010 tabulation-block TIGER shapefile set is currently retained locally.
- No 2010 block RCTX has yet been built.

Every retained PL source is recorded with its relative path, byte length, and
SHA-256 digest in `inventory.json`. The inventory remains `incomplete` until
the missing TIGER inputs are acquired and all contexts are built. It makes no
district-generation or legal-validity claim.

Regenerate with:

```text
python scripts/research/inventory_national_2010.py
```
