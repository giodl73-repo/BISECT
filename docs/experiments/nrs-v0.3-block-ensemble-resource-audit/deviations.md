# Deviations And Limitations

No frozen measurement parameter changed. Wilson ran before Kruskal, one runner
process at a time, with 50 ms sampling. Each scratch trace was deleted only
after exact normalized comparison passed.

The resource runs deterministically repeat the Stage 1 seed streams but are
classified as excluded resource replays. They are not additional statistical
draws and did not enter diagnostics, percentiles, or stopping decisions.

Peak RSS is the Windows OS-reported peak working set for the runner process.
The wrapper also recorded sampled resident memory; the OS peak is retained
because it captured transient final-serialization allocation between samples.
The result is author-machine evidence, not a portable performance guarantee.

NH/NM/GA projections scale Rhode Island wall time, peak RSS, and trace bytes by
audited block-count ratios. The frozen twofold compute/storage and 1.5-fold
memory margins are planning bounds, not validated scaling laws. A subsequent
protocol must retain hard abort ceilings and report any projection miss.
