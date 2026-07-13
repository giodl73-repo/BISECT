# Public Issue And Response Process

## Intake

Open one issue per reproducible defect or claim challenge. Include:

- artifact and version;
- source/input hashes;
- exact command;
- expected and actual result;
- environment;
- claim affected; and
- whether the issue is technical, statistical, legal, community, or custody.

## Triage

| Class | Initial response | Target disposition |
|---|---:|---:|
| Security/privacy | 2 business days | Coordinated process |
| Hash/replay failure | 5 business days | 30 days |
| Quantitative claim | 5 business days | 45 days |
| Legal/community challenge | 10 business days | Panel schedule |
| Documentation friction | 10 business days | Next documentation patch |

## Dispositions

- `confirmed-fixed`
- `confirmed-claim-withdrawn`
- `accepted-limitation`
- `environment-blocker`
- `not-reproducible`
- `disputed-with-record`

Every disposition links evidence and states whether prior artifacts are valid,
superseded, or withdrawn. Closing an issue does not delete the challenge.

## Stop Rule

Suspend the affected public claim when:

- a reference assignment cannot be reproduced;
- a package hash fails;
- a result cannot be regenerated;
- a legal summary contradicts operative text; or
- protected/private data were published improperly.
