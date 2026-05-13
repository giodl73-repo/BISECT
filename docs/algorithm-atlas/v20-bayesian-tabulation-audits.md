# V.20 Bayesian Tabulation Audits

## Mental Model

Bayesian audits model uncertainty about the true election outcome using prior
and posterior distributions. They can be informative and operationally useful,
especially for comparison with frequentist RLAs, but they are not automatically
risk-limiting unless calibrated to provide a risk limit.

RCOUNT should treat Bayesian audits as a parallel analytic transcript, not as a
replacement for risk-limiting claims.

## How RCOUNT Uses It

```text
sample observations -> prior/model -> posterior outcome probability -> analytic report
```

Bayesian audit support belongs in RCOUNT when the package needs to reproduce an
external Bayesian audit report or compare Bayesian and RLA evidence on the same
public artifacts.

## Step-By-Step Mechanics

1. Declare the prior and likelihood model.
2. Parse sample observations and population strata.
3. Compute or simulate posterior outcome probabilities.
4. Record credible intervals or posterior risk summaries.
5. If calibrated as risk-limiting, record the calibration rule.
6. Keep Bayesian conclusions separate from RLA pass/fail status.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `bayesian-tabulation-audit-v1` |
| `bayesian_prior_id` | prior family and parameters |
| `bayesian_likelihood_id` | sampling model |
| `posterior_winner_probability_ppm` | posterior probability for reported winner |
| `posterior_risk_ppm` | posterior probability that the reported outcome is wrong |
| `credible_interval` | posterior interval for margin or share |
| `simulation_seed` | seed for Monte Carlo implementation |
| `posterior_draws` | posterior simulation draws |
| `calibrated_risk_limit_ppm` | optional risk-limit calibration |

## Fixtures

- Synthetic analytic boundary fixture with prior/likelihood ids, deterministic
  seed, draw count, posterior winner probability, and posterior risk.
- Negative fixture where an impossible posterior risk is rejected.
- Calibration-boundary fixture that is informative but not risk-limiting.

## Current Implementation

RCOUNT can preserve and validate a Bayesian analytic transcript on
`AuditAlgorithmRun`:

- Bayesian runs must declare nonempty prior and likelihood ids;
- posterior winner probability and posterior risk must be valid ppm
  probabilities;
- deterministic Monte Carlo metadata can be recorded with `simulation_seed`
  and `posterior_draws`;
- assertions must be `BayesianOutcome`;
- non-Bayesian methods are rejected if they carry Bayesian posterior fields.

`rcount-audit` reports this method as a replay boundary:
the posterior analytics are recorded, but they are not treated as
risk-limiting replay.

## Claim Boundary

Bayesian probability is not the same claim as a frequentist risk limit. RCOUNT
must label Bayesian outputs as analytic unless the method includes an explicit,
reviewed risk-limit calibration.

## References

- Rivest/Shen Bayesian audit page: <https://people.csail.mit.edu/rivest/bayes/>
- Bayesian tabulation audits paper: <https://arxiv.org/abs/1801.00528>
