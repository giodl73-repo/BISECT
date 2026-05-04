# Revision Plan — prime-factor-redistricting

Stage: draft (not yet reviewed)

## Open TODOs before first review

### §4 Empirical work needed
- [ ] Run PFR for all 50 states at 2020 seat counts (implement k-way dispatch in runner.rs)
- [ ] Compute and report partisan outcomes per state (compare to B.7 MEC table)
- [ ] California full trajectory 48→53: generate maps, compute disruption scores
- [ ] PA 17-way cut: run and report outcome
- [ ] Disruption study: fraction of tracts changing district for 2010→2020 transitions
- [ ] Verify which 2020 seat counts are prime (confirm PA=17, VA=11, check others)

### §3 Algorithm
- [ ] Prove uniqueness of minimum k-way cut (or state as assumption)
- [ ] Formalize PFR-smooth algorithm

### §2 Related work
- [ ] Cohen 2021 and Hegeman 2020 citations need verification
- [ ] Add any legal scholarship on Art. I §2 vs §4 scope

### Implementation
- [ ] Add prime factorization dispatch to redist-cli runner.rs
- [ ] Add k-way METIS call path (already in library, needs CLI wiring)
- [ ] Script: scripts/b11_pfr_sweep.ps1
