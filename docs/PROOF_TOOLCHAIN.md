# Certified BISECT Proof Toolchain

## Pinned Tools

| Tool | Source | Pin |
|---|---|---|
| RoundingSat | `https://gitlab.com/MIAOresearch/software/roundingsat.git` | `d4edbf7908a9bb951fd181940919e0f3ac7ab1ee` |
| VeriPB | `https://gitlab.com/MIAOresearch/software/VeriPB.git` | `409a889bc23413e068af77361925b3d2b8e8cd66` (`SAT_competition_2023`) |

The tested RoundingSat static binary SHA-256 is:

```text
cf115250c7539000b39b950d53ddf9d1dfd4ca0d004caf52038a116e7efe3ed5
```

## Linux / WSL Setup

```bash
sudo apt-get update
sudo apt-get install -y python3 python3-pip python3-dev g++ libgmp-dev git

git clone https://gitlab.com/MIAOresearch/software/VeriPB.git
cd VeriPB
git checkout 409a889bc23413e068af77361925b3d2b8e8cd66
python3 -m pip install --user .
```

Download or build RoundingSat at the pinned source commit. If using the
official static artifact, verify its SHA-256 before execution.

## Smoke Replay

```bash
roundingsat \
  --lp=0 \
  --proof-log=population.pbp \
  population.opb

veripb \
  --requireUnsat \
  --stats \
  population.opb \
  population.pbp
```

Expected results:

```text
s UNSATISFIABLE
Verification succeeded.
```

## OPB Compatibility Rules

The pinned RoundingSat parser requires:

- variables named consecutively `x1` through `xN`;
- only `>=` and `=` relations;
- extended header fields `#equal` and `intsize`; and
- `--proof-log`, not the obsolete `--proof-output` option.

These requirements are enforced by `bisect-ilp::proof_backend`.

## LP Proof Compatibility

RoundingSat's LP-enabled proofs use the `rup` rule, which the historical
SAT-competition VeriPB pin does not support. The Rhode Island elite-two reduced
branch was therefore checked with VeriPB 3.0.2 at commit
`b1e507329fcb03efc5b85245d5e203bbfb6e55ef`. Its proof was streamed through a
named pipe and simultaneously gzip-compressed, avoiding a roughly 10 GB raw
proof file. This newer verifier pin currently applies only to that branch proof.

## Windows

Use WSL2 Ubuntu for the tested toolchain. Native Windows compilation is
possible with Visual Studio/vcpkg, but that path is not part of the current
verified custody.
