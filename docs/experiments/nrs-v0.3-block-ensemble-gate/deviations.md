# Deviations And Limitations

No governed chain was replaced, extended, or rerun under changed parameters.
No excluded preflight draw entered the governed analysis.

After both first-run traces completed, analysis initially failed while writing
JSON because a NumPy boolean was not converted to a native JSON boolean. The
conversion and a regression test were added. This did not alter either trace,
the frozen thresholds, the diagnostic calculations, or the stopping rule.

Peak memory was observed during periodic process monitoring but was not
captured by an instrumented peak-memory recorder. Consequently, this package
does not satisfy the separate expansion gate's requirement to report measured
peak memory, and no multi-State execution is authorized.

The two kernels target different sampled distributions. Their draws are not
pooled, and the sensitivity kernel is not an independent implementation. The
result does not establish mixing, independence, sampler equivalence, national
representativeness, neutrality, partisan or demographic fairness, VRA
compliance, polygon compactness, legal validity, or feasible-space coverage.
