# Requirement and acceptance traceability

Status: implementation baseline. Sprint 00 and Sprint 01 engineering tests are implemented; see the [foundation report](SPRINT_00_IMPLEMENTATION.md) and the [data report](SPRINT_01_IMPLEMENTATION.md). Later sprint tests remain planned. IDs refer to standalone sprint files. A Markdown checkbox alone does not satisfy a requirement. No Sprint 01 dataset is approved.

## Primary requirements

| Requirement | Implementation work | Acceptance evidence |
| --- | --- | --- |
| WJ-MSC starting profile, additional tissues | [02](sprints/02-ontology-profile.md) S02-01–10 | A02-01, A02-07–08 |
| Identity, expression, proteins, metabolism and culture | [02](sprints/02-ontology-profile.md) S02-02–06 | A02-01, A02-03–06 |
| Secretion, stress, viability, proliferation and senescence dynamics | [05](sprints/05-dynamic-simulation.md), [06](sprints/06-licensing-secretome.md) | A05-01–07; A06-01–06 |
| Differentiation potential with evidence/missingness | [02](sprints/02-ontology-profile.md) S02-03–04; [06](sprints/06-licensing-secretome.md) | A02-05–06; A06-09 |
| Separate cells and populations | [01](sprints/01-data-provenance.md), [02](sprints/02-ontology-profile.md), [05](sprints/05-dynamic-simulation.md) | A02-02; A05-07 |
| Donor, passage, batch, assay conditions and provenance | [01](sprints/01-data-provenance.md) S01-01–09 | A01-01–09 |
| Association, intervention, counterfactual semantics | [03](sprints/03-causal-graphs.md), [04](sprints/04-scm-counterfactuals.md) | A04-01–10 |
| Graphs, structural equations, time and uncertainty | [03](sprints/03-causal-graphs.md), [04](sprints/04-scm-counterfactuals.md), [05](sprints/05-dynamic-simulation.md), [09](sprints/09-fitting-uncertainty.md) | A03-08; A04-02–04; A05-01–10; A09-02, A09-05 |
| D-separation, confounding and applicable adjustment | [03](sprints/03-causal-graphs.md) S03-03–07; [04](sprints/04-scm-counterfactuals.md) S04-05 | A03-01–07; A04-06 |
| Abduction-action-prediction | [04](sprints/04-scm-counterfactuals.md) S04-06–07 | A04-02–04, A04-10 |
| Jev Choice, Score, Noul and calibration | [07](sprints/07-jev-routing.md) | A07-01–09 |
| Deterministic thresholds, authority and fallback | [07](sprints/07-jev-routing.md) S07-04–06 | A07-02–05, A07-07–08 |
| Separate local reasoning and optional hosted adapters | [08](sprints/08-reasoning-models.md) S08-01–04 | A08-01–02 |
| Hypotheses/reviews/interpretation/comparison proposals | [08](sprints/08-reasoning-models.md) S08-05–10 | A08-03–09 |
| No AI mutation of approved models or measurements | [03](sprints/03-causal-graphs.md) S03-08; [08](sprints/08-reasoning-models.md) S08-06–09 | A03-09; A08-03–07 |
| ODE solvers, stochastic state and interventions | [05](sprints/05-dynamic-simulation.md) | A05-01–10 |
| Rust-native chunked storage and versioned multi-omics | [01](sprints/01-data-provenance.md) S01-01–05 | A01-01–03, A01-08 |
| Transcripts, intracellular/secreted proteins and function remain distinct | [01](sprints/01-data-provenance.md), [02](sprints/02-ontology-profile.md), [06](sprints/06-licensing-secretome.md) | A01-01–03; A02-03; A06-05 |
| Donor splits, immutable manifests, accessions and licenses | [01](sprints/01-data-provenance.md) S01-06–10; [05](sprints/05-dynamic-simulation.md) S05-08–09 | A01-05–09; A05-09 |
| All required headless commands and JSONL | [00](sprints/00-foundation.md), [CLI contract](CLI_CONTRACT.md), [11](sprints/11-hardening-release.md) | A00-03–05; A11-02 |
| Stable schemas, seeds, exit codes, secure permissions | [00](sprints/00-foundation.md), [05](sprints/05-dynamic-simulation.md), [11](sprints/11-hardening-release.md) | A00-04–07; A05-06, A05-09; A11-03–07 |
| Analytically solvable mathematical verification | [03](sprints/03-causal-graphs.md), [04](sprints/04-scm-counterfactuals.md), [05](sprints/05-dynamic-simulation.md) | A03-01–07; A04-01–08; A05-01–04 |
| Independent donor/batch/passage/perturbation evaluation | [10](sprints/10-independent-evaluation.md) | A10-01–10 |
| Empirical/mechanistic baselines | [09](sprints/09-fitting-uncertainty.md), [10](sprints/10-independent-evaluation.md) | A09-04; A10-04–06 |
| Error, calibration, identification, uncertainty, latency and sources | [07](sprints/07-jev-routing.md), [09](sprints/09-fitting-uncertainty.md), [10](sprints/10-independent-evaluation.md), [11](sprints/11-hardening-release.md) | A07-06; A09-04–06; A10-05–09; A11-05, A11-08 |

## Non-negotiable release gates

| Gate | Pass condition | Tests |
| --- | --- | --- |
| N1: Rust-only default | Default runtime builds/runs without Python or JavaScript, with audited dependency features | A00-01–02; A11-01, A11-07 |
| N2: Parameter honesty | Every biological parameter is sourced with context or explicitly illustrative | A02-04; A06-06 |
| N3: Distinct causal levels | Conditioning, intervention and counterfactual abduction remain different operations | A04-01–04, A04-07–10 |
| N4: Offline independence | Missing AI credentials/services do not stop simulation/explanation | A06-07; A07-03, A07-07–08; A08-02; A11-06 |
| N5: Complete causal metadata | Every causal success/refusal includes provenance, model revision, assumptions and uncertainty status | A04-09; A11-05 |
| N6: Honest unsupported outcomes | Unsupported/non-identifiable numeric requests produce no invented effect | A03-06–07; A04-05, A04-08; A05-10 |
| N7: Headless agent use | Every required function available without GUI/TTY through stable CLI/JSONL | A06-08; A11-02 |
| N8: Evidence-based physiological claims | No positive physiological claim without independent benchmark support for its stated domain | A09-10; A10-07–10; A11-09–10 |

## Initial demonstration gates

The [single-terminal demonstration](DEMO.md) must initialize the illustrative profile, inspect/validate graphs, observe modeled state, simulate over time, apply an intervention, compute a supported counterfactual, explain actual mathematics and replay immutable manifests. Owners: S02-09, S03-09, S04-09–10, S05-09 and S06-09. Final gate: **A06-07 and A06-08**.

## Planning completion versus build completion

The planning document set is complete. Full software completion requires the remaining implementation, experimental evaluations and release gates with actual evidence and sign-off. Sprint 00 establishes engineering functionality only; all independent scientific sign-offs remain pending.
