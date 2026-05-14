# Fork Context: Pulse 01

Execute the J Divisor Method Evidence pulse end to end.

## Scope

- Add a minimal package fixture for Webster, Adams, Jefferson, and shared
  divisor Alabama-paradox immunity.
- Add verifier coverage in `bisect-apportion`.
- Update J.2--J.5 papers and docs.
- Rebuild PDFs, validate, and commit.

## Decision

This slice needs fixture/docs plus a small verifier helper in
`bisect-apportion`; it does not require a new public `rctx-io`-style crate or
standalone CLI because the consumer is the J-track evidence verifier.
