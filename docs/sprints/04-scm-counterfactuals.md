# Sprint 04 — SCMs, interventions and counterfactuals

Status: **Not started**. Accountable owner: **Causal inference lead** (named assignee pending). Reviewers: independent causal methods reviewer and numerical engineer. Milestone: **M2**, completion.

## Objective

Implement mathematically distinct observational, interventional and counterfactual computation for explicitly supported SCM families. Reject unsupported or non-identifiable numeric requests instead of replacing them with guesses.

## Dependencies and entry criteria

- Sprints 01 and 03 accepted; datasets, graph certificates and source/assumption records available.
- [Specification causal contract](../SPECIFICATION.md) and [demo mathematics](../DEMO.md) reviewed.
- Exact/discrete and linear-Gaussian scope documented, including numerical and resource limits.

## Actionable backlog

- [ ] **S04-01 — Structural mechanisms** (Causal): Implement `M=(U,V,F,P(U))`, bounded expression/registered-function mechanisms, parameter units and explicit exogenous joint distributions. Validate parent agreement and separate measurement errors from state variation.
- [ ] **S04-02 — Forward evaluation** (Causal): Evaluate acyclic mechanisms in stable order; implement finite discrete enumeration and linear-Gaussian joint moments. Include latent/common exogenous variables rather than silently assuming independence.
- [ ] **S04-03 — Association service** (Statistics): Implement declared empirical summaries/conditional distributions and SCM conditioning as different sources. Report counts, missingness, units, conditioning set and uncertainty. Do not label a regression coefficient as an effect by default.
- [ ] **S04-04 — Structural intervention** (Causal): Replace targeted equations with constants or supported mechanisms, remove incoming edges and preserve other mechanisms/exogenous background. Support outcome distributions and explicit intervention contrasts; validate treatment values and units.
- [ ] **S04-05 — Identified estimation** (Statistics + Causal): Evaluate supported discrete adjustment/front-door functionals from eligible probability tables. Check support/positivity and missing strata. Keep symbolic identification success separate from finite-data estimation eligibility.
- [ ] **S04-06 — Abduction** (Causal): Implement exact discrete posterior inference and conditional Gaussian inference for full/partial evidence. Handle noisy observation nodes, covariance rank, inconsistent evidence and resource limits; retain a posterior artifact over exogenous variables.
- [ ] **S04-07 — Action and prediction** (Causal): Apply interventions to the abduced model and propagate the same posterior/background into the counterfactual world. Preserve joint factual/counterfactual coupling and report posterior uncertainty. Never replace abduction with seed reuse.
- [ ] **S04-08 — Refusal and basis contract** (Platform): Enforce explicit `observational_identification` versus `specified_scm` basis. No silent model-based fallback when an observational causal query fails. Return structured reasons/certificates with no numeric outcome on refusal.
- [ ] **S04-09 — Commands and deterministic explanation** (Platform): Implement `intervene`, `counterfactual`, association requests under `observe`, and `explain`. Explanations derive from equations, identification and abduction/action traces; no language model is required.
- [ ] **S04-10 — Demo fixtures and analytic suite** (Causal): Ship the toy SCM and synthetic evidence/request files in the demo bundle. Add non-identifiable observationally equivalent models, front-door tables and unsupported nonlinear cases. Label fixtures as mathematical, not physiological evidence.

## Deliverables

Rust SCM engine, exact and Gaussian inference, typed query results, posterior/trace artifacts, structural interventions, identified estimators, refusal handling and a complete analytic acceptance suite.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A04-01 | `C=Uc`, `X=C+Ux`, `Y=2X+3C+Uy`; independent standard normals | `E[Y|X=x]=3.5x`, `E[Y|do(X=x)]=2x`; query types remain distinct |
| A04-02 | Evidence `C=1,X=2,Y=8`, action `do(X=0)` | Abduced residuals are `(1,1,1)` and counterfactual Y is 4 |
| A04-03 | Same model, evidence only `X=2`, action `do(X=0)` | Counterfactual Y has mean 3 and variance 5.5; no fabricated unique residual |
| A04-04 | Noisy measurement evidence and impossible discrete evidence | Correct posterior for supported noise model; zero-probability discrete evidence rejected |
| A04-05 | Binary models `X=U,Y=X` and `X=U,Y=U`, with fair U | Same observed distribution, different do-outcomes; observational effect query refused without extra assumptions |
| A04-06 | Exact discrete intervention versus enumerated graph-adjustment/front-door formula | Matching probabilities on identifiable fixtures; invalid support prevents estimation |
| A04-07 | Intervention on a parented variable | Incoming edges removed; original immutable model unchanged; descendants update correctly |
| A04-08 | Unsupported nonlinear abduction, invalid covariance and excessive state space | Explicit unsupported/numerical/resource status, no numeric causal answer |
| A04-09 | CLI/JSONL successes and refusals | Model revision, provenance, assumptions, identification and uncertainty fields always present |
| A04-10 | Deterministic explanation of A04-02 | Shows factual evidence, residual inference, modified equation and resulting 4 without AI |

Analytic assertions use exact arithmetic where available and declared numerical tolerances otherwise. Gaussian conditioning on an exact continuous observation uses a regular conditional distribution; it must not be rejected merely because a singleton has probability zero.

## Evidence to collect

Store `evidence/sprint-04/<run-id>/` with hand-derived expectations, test outputs, discrete enumerations, posterior summaries, intervention graph diffs, refusal reports and explanation transcripts. State `synthetic_verified` with the exact supported families. Current evidence: **none**.

## Limitations and risks

Counterfactual validity depends on structural assumptions and a specified exogenous distribution; observations alone generally do not fix cross-world coupling. This sprint does not solve arbitrary nonlinear/dynamic abduction. Correct toy results do not validate WJ-MSC behavior. Positivity failures must not be repaired with undocumented smoothing or extrapolation.

## Completion sign-off

- [ ] S04-01 through S04-10 completed; A04-01 through A04-10 passed.
- [ ] Independent reviewer confirmed the analytic derivations and refusal semantics.
- [ ] No synthetic result is described as experimental physiological validation.

Engineering approver: **pending**. Causal reviewer: **pending**. Software/model revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
