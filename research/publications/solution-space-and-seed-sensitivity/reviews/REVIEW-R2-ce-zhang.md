# Review Round 2: R-30 Ce Zhang
**Paper**: Solution Space of Minimum-Edge-Cut Redistricting
**Date**: 2026-05-01
**Score**: 3.5 / 4

---

## P1 Item Resolution

**P1-C (Election data citation)**: Fixed. VEST/Fekrazad DOI added, precinct-to-tract interpolation methodology noted. This is sufficient for the claims made.

**P2-A (Exact TIGER perimeters)**: Fixed. The switch to exact TIGER polygon perimeters via pyproj EPSG:5070 projection is correct. PP values now in [0,1]. The implementation reads .shp files directly without a GIS library dependency. External perimeters for interior tracts are correctly 0 (no state-boundary contribution). The median_ext = 0 finding for most tracts is expected and correct.

---

## New Strengths

**S-1 (National seat totals with AK/HI)**: The addition of the 229D/206R national estimate (vs enacted 222D/213R) is the paper's most politically salient finding and is now stated clearly. The +7 Democrat seats figure will be widely cited.

**S-2 (False floor documentation)**: The data quality analogy is direct: just as ML results can be "data artifacts" that disappear with more data, convergence claims can be "seed artifacts" that disappear with more seeds. The GA 7D → 6D → 7D arc is the clearest demonstration.

---

## Remaining Concerns

**R-1 (PA table still 100-seed data)**: The main PA evaluation table (Table 2) uses 100-seed data while the rest of the paper cites 1,100-seed data. This is a data consistency problem. The table header says "Pennsylvania, 100 seeds" but the convergence paragraph references 1,100-seed results. For a paper about convergence, having inconsistent seed counts in the same section is a credibility issue. Update Table 2 to 1,100 seeds.

**R-2 (WI GMPP not yet certified)**: The paper concludes that "both MEC and GMPP agree on 3D for WI." But GMPP last improved at seed 930 with 69-seed tail — not certified by the 300-seed standard. The agreement claim should be qualified: "provisionally agree (GMPP not yet certified)."

**R-3 (Alaska election data gap)**: The AK/HI note in §4.5 says these are excluded "due to missing tract-level election data." AK's exclusion is explained, but the at-large seat outcome (Republican, Don Young 2020) is stated as a "best-guess" without citation. Add a footnote with the actual 2020 Alaska at-large result (R, Don Young won with 54.1%) and Hawaii (D, both seats). Precision matters for a paper with legal implications.

---

## Verdict

The exact perimeters and election citation fix the main data quality concerns from Round 1. PA table consistency and WI GMPP caveat are easy editorial fixes. Score rises to 3.5. Ready for submission after R-1.
