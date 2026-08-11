# Terminal Expansion Failure

The frozen `nrs-v0.3-block-ensemble-expansion-v1` gate is closed and failed.

On 2026-08-11, the sixth primary (`GA:kruskal`) began in the frozen order with
the registered parameters. After 602.30 runner seconds, the Rust runner exited
with code 1 and reported:

```text
Error: There is not enough space on the disk. (os error 112)
```

The enforcing wrapper retained
`resource-primary-ga-kruskal.json`, the zero-byte scratch output, and the
terminal ledger entry. Its monitor recorded 1,401,757,696 peak RSS bytes,
below the 2,415,919,104-byte process ceiling. A host audit immediately after
failure found only 72,396,800 free bytes on the volume.

The protocol states that exceeding any resource ceiling terminates the active
process, retains a failure record, and closes the expansion without retry.
Accordingly, GA Kruskal was not retried and no governed replay was started.
Five primary State/kernel traces remain retained, but the expansion gate did
not pass and the registered completed-expansion claim is unavailable.

After preserving the failure evidence, 4,669,167,840 bytes of generated
`target/debug` Cargo artifacts were removed to permit documentation and Git
custody. The release runner and all governed evidence remained intact.
