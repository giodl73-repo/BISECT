# Certified Versus METIS Comparison

The path-8 root package compares three assignments on the identical split
instance:

- the bounded certified optimum;
- vendored METIS 5.1.0 with seed 42; and
- a deliberate connected but population-suboptimal control.

METIS selects the same assignment and objective as the certified oracle on this
simple fixture. Certification therefore adds proof strength, not a better cut,
for this case. The control demonstrates that the comparison detects a genuinely
worse connected assignment.

## Rebuild And Verify

```powershell
cargo run -p bisect-cli --example certified_vs_metis_path8 -- `
  docs\examples\certified-comparison\path8-root

python scripts\research\verify_certified_vs_metis.py
```

## Claim Boundary

This is one bounded synthetic root split. It does not establish national
runtime, compactness, partisan, community, or final-plan superiority.
