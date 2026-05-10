---
journal: The Districting Review
volume: 1
article: 3
title: "Seed Sensitivity Is Bounded"
status: provisional-draft
---

# Seed Sensitivity Is Bounded

The next question is whether the defense collapses when the seed changes.

The answer, in the source material, is no: the relevant sensitivity is bounded
and scoped. That is the right public claim. It says the algorithm is not a
black box with uncontrolled drift, but it does not say the output is invariant
under every possible parameter choice.

This distinction matters. A bounded-sensitivity result is a strong defense for a
practitioner audience because it shows the output space is constrained. It is
not the same thing as proving that no adversary can ever find a better
configuration.

## Source Basis

This draft relies on [B7] and [B17].

## Review Boundary

Keep the scope exact. The audience should know which districts, which census
vintage, and which metric the bounded claim covers.
