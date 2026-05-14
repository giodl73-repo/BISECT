# Fork Context: Pulse 01

Execute the M.3 ACS housing evidence slice end to end.

## Decision

This slice needs code plus fixture/docs: the ACS fetch path existed, but the
housing-character weight mode and package verifier were missing. No new
standalone CLI verifier is required; Rust test coverage is the consumer.
