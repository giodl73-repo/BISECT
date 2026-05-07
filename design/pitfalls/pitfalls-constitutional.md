# Constitutional Pitfalls (CP-01..CP-02)

Structural vulnerabilities that produce legally invalid redistricting outputs. BOUNDARY owns these. The standard: constitutional requirements must be structurally impossible to violate, not merely unlikely.

---

## CP-01: Policy optimization competes with constitutional constraints

**Pattern:** When an optimizer has latitude to trade between a constitutional constraint (population balance) and a policy goal (minority concentration), it will make that trade whenever locally beneficial. The constitutional constraint is not structurally enforced — it is merely a preference among preferences.

**Domain:** Any redistricting algorithm where constitutional requirements (±0.5% population balance) and policy goals (VRA compliance, compactness) are expressed as competing objectives in the same optimization. Multi-objective optimizers explore tradeoffs by design — that is the point of multi-objective formulation. Constitutional constraints must not participate in any tradeoff.

**Structural solution:** Constitutional constraints live in the optimizer's enforcement layer (ufactor), not in its objective function. Policy goals live in the graph structure (edge weights), which shapes but does not override enforcement. The optimizer has one objective and one enforcement constraint; they cannot trade.

**Status:** SOLVED
**Proved by:** V4: 44/44 multi-district states pass ±0.5% population balance
**Test:** `tests/integration/test_vra_pipeline_balance.py`

---

## CP-02: Algorithm operating outside its valid input domain produces silent failure

**Pattern:** An algorithm designed for inputs in domain D receives an input at the boundary of D (or outside it) and produces output that is technically valid in form but constitutionally invalid in substance — or crashes, leaving no output. The caller doesn't know because no error was raised.

**Domain:** Any redistricting algorithm applied to a state with unusual characteristics (1 district, island geography, extreme tract topology). The algorithm was designed and tested for typical cases; edge cases reveal assumptions baked into the design.

**Structural solution:** Explicit domain validation before algorithm entry. For inputs outside the algorithm's valid domain, fail early with a clear message and fall back to a known-valid behavior. Never let an out-of-domain input reach the algorithm's internals.

**Status:** SOLVED for single-district states (explicitly handled before VRA block)
**Test:** `tests/integration/test_vra_pipeline_balance.py::TestVRACodePathIntegrity`
