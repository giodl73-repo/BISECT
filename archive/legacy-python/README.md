# Legacy Python Custody

This directory contains retired Python experiment machinery preserved only
for historical custody. It is not an active code path and must not be run or
extended.

The original relative layout is retained because historical operational-tree
manifests recorded builder paths under `scripts/research`. `bisect-ops verify`
uses this archive as a legacy fallback when checking those recorded hashes.
Some early manifests reference source bytes that did not survive in Git; those
packages correctly fail the source-custody check rather than silently
weakening verification.

The State block RCTX builder was retired after the Rust implementation matched
Rhode Island's complete unit universe, populations, edge set, edge kinds, and
weights exactly.
