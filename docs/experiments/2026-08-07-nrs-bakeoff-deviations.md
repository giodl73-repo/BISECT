# NRS v0.3 Bakeoff Deviations

## Deviation 01 - Comparator Session Identity

**Observed before accepted result publication:** The preregistered comparator
path `data/2020/baseline/2020/us_cd118_2020.parquet` contains Rhode Island
polygons whose internal `CDSESSN` value is `116`, not `118`.

The source therefore failed the intended comparator identity even though its
filename contains `cd118`. The generated candidate metrics were discarded and
are not evidence.

**Disposition:**

- retain the rejection in this deviation record;
- use the already-downloaded official Census state archive
  `data/enacted_districts/tl_2020_44_cd118.zip`;
- require `STATEFP20=44`, `CDSESSN=118`, and exactly two `CD118FP` values before
  block projection; and
- reject future filename/session disagreements before metric computation.

The replacement is based solely on source identity and occurred before an
accepted package was published. It was not selected based on split or overlap
outcomes.

## Deviation 02 - Uniform Water-Only Block Exclusion

The first national Tier 1 execution failed for Connecticut, Illinois, and New
Hampshire because their official CD118 archives contain a `ZZ` polygon named
`Congressional Districts not defined`. In each archive that polygon has
`ALAND20=0` and consists only of water.

**Disposition:** Amend the atomic-universe rule uniformly for every State to
retain only 2020 tabulation blocks with `ALAND20 > 0`, then exclude non-numeric
comparator polygons before projection. This is an input-semantics correction,
not an outcome-selected replacement. Any retained block without exactly one
numbered comparator district still fails the package.
