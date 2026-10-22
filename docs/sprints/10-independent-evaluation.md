# Sprint 10 — Independent biological benchmarks

Status: **Not started**. Accountable owner: **Evaluation lead** (named assignee pending). Reviewers: independent statistical reviewer, MSC scientist and data steward. Milestone: **M6**.

## Objective

Evaluate actual biological predictions against independent, eligible experimental observations, with honest uncertainty and baseline comparisons. Complete evaluation even when models fail; permit positive claims only where predeclared evidence supports them.

## Dependencies and entry criteria

- Sprints 01, 06 and 09 accepted; data rights/provenance, observation models, frozen fits and baseline artifacts available.
- Real independent data with documented donor, batch, passage and perturbation context obtained and eligible. No candidate accession becomes an approved benchmark without review.
- Evaluation steward locks the independent dataset and plan before candidate evaluation. Record any prior developer exposure to these outcomes and associated limitations.
- If suitable data do not exist or are inaccessible, report the blocker and continue preparatory work; do not mark this sprint complete.

## Actionable backlog

- [ ] **S10-01 — Eligibility audit** (Steward): Finalize dataset accessions/versions, licenses, tissue/culture compatibility, measurement types, donor counts, biological/technical replicates, perturbation design and exclusions. Identify duplicates across source studies and training data.
- [ ] **S10-02 — Locked evaluation plan** (Evaluation): Predeclare primary/secondary targets, time windows, estimands, units, metrics, acceptance margins, calibration criteria, uncertainty methods, multiplicity treatment and sample-size/power rationale. Hash the plan before evaluating held-out outcomes.
- [ ] **S10-03 — Donor-independent suite** (Evaluation): Hold out complete donors with all cells/samples/replicates; include a genuinely external source or independently acquired cohort. Keep training, calibration and test membership immutable.
- [ ] **S10-04 — Batch/passage/perturbation suites** (Evaluation): Test unseen batches, held-out passage ranges and held-out interventions/doses/times. Maintain separate generalization claims and crossed group holdouts where feasible. If passage or batch is inseparable from donor/study, report confounding rather than claiming independent isolation.
- [ ] **S10-05 — Observation alignment** (Biology): Compare predictions and measurements through matching units, compartments, collection windows, cell counts, volume and assay operators. Report unavailable targets rather than translating transcripts to secretion or secretome to potency without a validated mechanism.
- [ ] **S10-06 — Frozen baseline evaluation** (Evaluation): Run all selected sophisticated models and simple empirical/mechanistic baselines on identical eligible examples. No retuning after test inspection. Preserve failures, missing predictions and compute cost.
- [ ] **S10-07 — Metrics and uncertainty** (Statistics): Report per-target MAE/RMSE and appropriate probabilistic scores, calibration, interval coverage/width, donor-cluster uncertainty, per-domain/subgroup results, sample sizes, out-of-domain flags and latency/memory. Distinguish biological variability from technical replicates.
- [ ] **S10-08 — Causal benchmark audit** (Causal): Use perturbation data only under its documented assignment/identification assumptions. Report causal-effect error only where the design supports it. Do not claim unit-level counterfactual ground truth when only one potential outcome is observed; validate observable implications and label the rest untestable.
- [ ] **S10-09 — Claims register/model cards** (Biology + Evaluation): Map every proposed physiological claim to endpoint, tissue, donor/culture domain, horizon, perturbation, uncertainty and supporting benchmark. Classify supported, unsupported, inconclusive and contradicted claims; retain negative evidence.
- [ ] **S10-10 — Independent reproduction** (Reviewer): Have a reviewer rerun frozen manifests without training modifications, verify artifact integrity and reconcile results. Publish a rights-compatible evaluation package with source references and aggregate reports, not restricted raw records.

## Deliverables

Locked eligibility and evaluation manifests; independent donor, batch, passage and perturbation reports; complete baseline comparisons; causal-identification audit; bounded claims register; and independent reproduction evidence. These are experimental results, not placeholders filled with synthetic scores.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A10-01 | Audit accession/license/source and donor overlap | All evaluated records eligible; no known training/test donor or duplicate-source leakage |
| A10-02 | Verify evaluation-plan and frozen model hashes | Plan predates test evaluation; no post-hoc tuning disguised as confirmatory testing |
| A10-03 | Execute all four domain holdout suites | Donor, batch, passage and perturbation outcomes reported; missing/confounded suite blocks its claim and full required coverage |
| A10-04 | Evaluate every candidate and baseline on the same eligible records | Complete, comparable metrics and failures; no cherry-picked exclusions |
| A10-05 | Recompute metrics and donor uncertainty independently | Results reproduce within declared tolerances and reported sample sizes |
| A10-06 | Inspect calibration, interval coverage and domain failures | Actual measured behavior reported, including undercoverage or baseline superiority |
| A10-07 | Ask for causal-effect validation from purely observational prediction scores | Refused unless a valid identification/design basis is supplied |
| A10-08 | Ask for individual counterfactual accuracy without both potential outcomes | Metric marked unavailable; no invented ground truth |
| A10-09 | Review physiological claim against predeclared criteria | Only supported, scoped claims admitted; negative/inconclusive evaluation remains a valid recorded outcome |
| A10-10 | Run independent reviewer reproduction | Same inputs, outputs and conclusions with linked provenance and review signature |

Performance success and sprint completion are distinct: execution of adequate predeclared benchmarks may complete the sprint even if all sophisticated models underperform. Missing required data or an unexecuted holdout suite is not a completed benchmark.

## Evidence to collect

Store `evidence/sprint-10/<run-id>/` with dataset/split/plan hashes, source/license audit, frozen model and baseline versions, actual metric tables, uncertainty/calibration reports, causal design audit, claim decisions and reviewer reproduction. Restricted observations remain in authorized storage. Current evidence: **none**.

## Limitations and risks

Independent sample sizes may be too small for decisive claims. Batch, passage and donor can be confounded. Unseen perturbations may be outside supported model domains. A strong average score can hide donor failures. No retrospective benchmark grants universal tissue transfer or clinical validity. Lack of data is a real external blocker, never a reason to substitute synthetic validation.

## Completion sign-off

- [ ] S10-01 through S10-10 completed; A10-01 through A10-10 passed with real eligible observations.
- [ ] All required holdout suites executed; unresolved gaps explicitly prevent completion where applicable.
- [ ] Claims register reflects actual evidence, including failures and inconclusive outcomes.

Engineering approver: **pending**. Independent statistical/MSC reviewers: **pending**. Software/model/dataset revisions: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
