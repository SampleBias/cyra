# Sprint 09 — Model fitting, uncertainty and hybrid prediction

Status: **Not started**. Accountable owner: **Statistical modeling lead** (named assignee pending). Reviewers: causal inference lead and independent evaluation engineer. Milestone: **M5**.

## Objective

Fit supported mechanisms in Rust, quantify explicitly scoped uncertainty, and compare learned or hybrid models against simple baselines without data leakage or altered causal meaning.

## Dependencies and entry criteria

- Sprints 01 and 04–06 accepted; typed assays, structural models, dynamics and observation operators available.
- Training/validation datasets are eligible and separate from the untouched independent evaluation set.
- Synthetic fixtures may prove fitting correctness; biological fit quality requires actual measured data and must be labeled separately.
- Optional AI integration is not a dependency of fitting or uncertainty computation.

## Actionable backlog

- [ ] **S09-01 — Fit-plan schema** (Statistics): Define target parameters, fixed assumptions, bounds/priors, observation likelihood, censoring/missingness handling, objective, optimization budget, seeds and train/validation split hashes. Require parameter units and declared supported model family.
- [ ] **S09-02 — Rust fitting implementation** (Statistics + Numerical): Implement bounded fitting for a modest identifiable mechanism family, using weighted residual likelihood or an appropriate supported likelihood. Check convergence, multiple starts, gradient accuracy if used, and solver failures inside the objective.
- [ ] **S09-03 — Parameter identifiability** (Statistics): Add sensitivity/rank or profile-objective diagnostics for selected models. Report flat/confounded parameter directions and retain uncertainty. Keep parameter identifiability distinct from causal query identification.
- [ ] **S09-04 — Empirical baselines** (Statistics): Implement training-only mean/constant and regularized regression baselines with the same targets/features and missing-data policy. Save preprocessing and regression parameters as immutable artifacts.
- [ ] **S09-05 — Mechanistic baselines** (Numerical): Implement persistence/decay and the simplest relevant production/clearance baseline. Compare on the same time grid, initial information and assay observation operator as the fitted mechanism.
- [ ] **S09-06 — Uncertainty methods** (Statistics): Implement at least a supported parameter uncertainty method and predictive propagation, such as donor-cluster bootstrap plus observation-noise sampling. Record what uncertainty each interval includes and excludes; do not label bootstrap samples as a Bayesian posterior.
- [ ] **S09-07 — Hybrid candidate** (Statistics + Causal): Implement an optional residual/empirical correction candidate with typed feature scope and constraints. If used only for prediction, explicitly disallow causal/counterfactual calls through it. Causal use requires a reviewed structural equation and validated exogenous interpretation.
- [ ] **S09-08 — Leakage barriers** (Evaluation): Fit normalization, feature selection, imputation if any, calibration and hyperparameters only within training/validation partitions. Preserve missing originals; save derived imputed data separately. Freeze the selected model before external test access.
- [ ] **S09-09 — Diagnostics and calibration** (Evaluation): Report prediction errors, interval coverage/width, suitable probabilistic scores, residual patterns, subgroup sizes, sensitivity, out-of-domain status and failure rates. Use donor-level uncertainty rather than counting cells as independent donors.
- [ ] **S09-10 — Commands and artifacts** (Platform): Implement `fit`, `evaluate` for mathematical/internal validation, and JSONL services. Store fit plan, parameter provenance, uncertainty artifacts, baselines, training history, convergence status and model revision. A failed fit cannot replace the approved model.

## Deliverables

Rust fit pipeline, frozen plan schema, parameter/uncertainty artifacts, empirical and mechanistic baselines, optional hybrid candidate, leakage tests and internal validation report with explicit evidence tier.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A09-01 | Fit known parameters in an identifiable synthetic mechanism | Recovery within predeclared noise/tolerance bounds; actual deviations reported |
| A09-02 | Fit deliberately non-identifiable parameter combination | Diagnostic identifies ambiguity; no falsely precise unique estimate |
| A09-03 | Introduce extreme values only in held-out outcomes | Training preprocessing, chosen parameters and model selection remain unchanged |
| A09-04 | Compare constant, regression, persistence and mechanism baselines | Same split/targets/input availability; complete score table includes baseline wins |
| A09-05 | Repeated synthetic interval-coverage experiment | Coverage assessed using a predeclared finite-sample tolerance; source of uncertainty labeled |
| A09-06 | Donor-cluster bootstrap with many cells per donor | Donors, not cells, are resampling units; uncertainty report states effective donor count |
| A09-07 | Censored/missing assay values and invalid likelihood inputs | Supported likelihood handles them explicitly or rejects; no zero imputation |
| A09-08 | Hybrid predicts an invalid physical state or receives a causal request without structural approval | Invalid outputs/requests rejected or constrained by an explicitly documented mechanism |
| A09-09 | Repeat fitting with frozen inputs/seed, then force nonconvergence | Reproducible reference fit; failed run has no approved successful parameter artifact |
| A09-10 | Request biological validation status based only on synthetic recovery | Remains synthetic/internal status; independent physiological claim unavailable |

## Evidence to collect

Store `evidence/sprint-09/<run-id>/` with fit plans, split hashes, parameter recovery/identifiability reports, baseline score tables, leakage tests, uncertainty diagnostics, convergence logs, latency and data/parameter provenance. Label every result synthetic versus experimental. Current evidence: **none**.

## Limitations and risks

Sparse assays may leave many parameters unidentifiable. Parameter intervals omit model-form uncertainty unless explicitly modeled. Good predictive fit does not identify interventions or counterfactuals. Hybrid corrections can destroy mechanistic semantics if applied indiscriminately. No test-set tuning is allowed; poor performance must remain visible.

## Completion sign-off

- [ ] S09 mandatory tasks completed, including comparison/reporting of the hybrid scope decision; A09-01 through A09-10 passed.
- [ ] Parameter uncertainty, leakage controls and causal-use boundaries reviewed.
- [ ] Frozen candidate artifacts are ready for independent evaluation without further tuning.

Engineering approver: **pending**. Statistical/causal reviewer: **pending**. Software/fit revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
