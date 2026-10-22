# Sprint 02 — MSC ontology and illustrative reference profile

Status: **Not started**. Accountable owner: **Computational biology lead** (named assignee pending). Reviewers: data engineer and MSC domain reviewer. Milestone: **M1**, completion.

## Objective

Create a typed human WJ-MSC representation with explicit cell/population boundaries and an illustrative reference bundle. Represent all requested biological domains even where behavior is not yet modeled.

## Dependencies and entry criteria

- Sprints 00–01 accepted, including missingness and provenance types.
- [BIO-01 through BIO-04](../REFERENCES.md) assigned for contextual review; missing full-text access recorded.
- Profile schema and future numerical-state requirements reconciled before fixing field names.

## Actionable backlog

- [ ] **S02-01 — Cell and population types** (Biology + Platform): Implement stable IDs, donor links, single-cell state, population membership/distributions, live/dead counts, aggregation weights and sampling context. Bulk measurements remain population-level records.
- [ ] **S02-02 — Culture context** (Biology): Represent tissue, passage/history, medium and supplements, oxygen, temperature, duration, density, vessel/volume and batch. Use explicit unknowns; avoid universal culture defaults presented as measured conditions.
- [ ] **S02-03 — State ontology** (Biology): Define identity, transcripts, intracellular proteins, secretion, metabolism, viability, proliferation, senescence, oxidative stress, inflammatory licensing and lineage-specific differentiation potential. Each field has a domain, units, provenance and modeled/unmodeled status.
- [ ] **S02-04 — Identity evidence** (Biology): Curate characterization rules using historical and applicable newer sources. Represent surface-marker and functional/differentiation assay evidence contextually; marker expression alone cannot certify potency or universal identity. Record literature scope and conflicts.
- [ ] **S02-05 — Units and observation operators** (Platform + Biology): Implement conversions with explicit dimensional checks. Separate transcript counts/normalized expression, intracellular abundance/activity, secretion rate, extracellular concentration and functional responses. Link observable assays to state through declared observation operators.
- [ ] **S02-06 — Parameter registry** (Biology): Require name, value/distribution, units, admissible range, mechanism role, origin, source/extraction context or illustrative rationale, uncertainty and reviewer. Reject unsourced numeric biological defaults that lack an illustrative label.
- [ ] **S02-07 — WJ illustrative bundle** (Biology): Create `wj-msc-illustrative` with synthetic donor/batch IDs, all required state domains, parameter annotations and a model card. Include no fictitious experimental measurements; placeholders use missing/unmodeled status.
- [ ] **S02-08 — Tissue extensibility** (Platform): Define versioned profile loading/validation so another tissue profile can be added without changing core enum logic. Test with a synthetic extension; do not claim another tissue is calibrated.
- [ ] **S02-09 — CLI and demo assets** (Platform): Implement `init`, profile inspection under `observe`, and JSONL equivalents. Materialize a self-describing profile bundle; reserve documented names for the later [demo assets](../DEMO.md). Never ship absent mechanism requests as successful simulations.
- [ ] **S02-10 — Model card and review** (Biology): State intended use, tissue/culture scope, evidence tier, unmodeled biology, unsupported assays and parameter provenance coverage. Distinguish a reference starting point from a consensus quantitative MSC state.

## Deliverables

Cell/population/culture/profile schemas, units and observation types, parameter registry, illustrative WJ profile, identity evidence register, profile loader and complete profile model card.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A02-01 | Initialize and round-trip the WJ bundle | All required domains and lineage/provenance metadata survive |
| A02-02 | Try to assign a bulk RNA measurement to each cell as observed data | Reject or require an explicitly derived allocation artifact; preserve original scope |
| A02-03 | Convert rate to concentration without cell number, volume or time | Reject insufficient dimensional/context information |
| A02-04 | Supply unsourced biological parameter and then label it illustrative | First rejected; second accepted with visible origin/limitation |
| A02-05 | Query missing differentiation/functional evidence | Return missing/unsupported, never invented capability or numeric potency |
| A02-06 | Inspect marker data from different culture/licensing contexts | Context retained; no unconditional identity/potency inference |
| A02-07 | Load a synthetic second-tissue bundle | Same public APIs work without changes to core cell code |
| A02-08 | Inspect profile in JSONL and human modes | Illustrative state, absent measurements and provenance are unmistakable |

## Evidence to collect

Store `evidence/sprint-02/<run-id>/` with schema validation, unit-conversion tests, provenance coverage audit, model card, source review notes and human/JSONL inspection transcripts. All synthetic identifiers are visibly synthetic. Current evidence: **none**.

## Limitations and risks

The ontology records metabolism and differentiation before detailed dynamics exist. Identity criteria depend on context and evolving definitions. The initial bundle contains no measured donor reference and cannot establish viability, potency or predictive accuracy for a real sample. Illustrative proxies must not masquerade as molecular measurements.

## Completion sign-off

- [ ] S02-01 through S02-10 completed; A02-01 through A02-08 passed.
- [ ] Biological state boundaries, units and source/illustrative labels reviewed.
- [ ] Model card explicitly states unvalidated physiological status.

Engineering approver: **pending**. MSC scientific reviewer: **pending**. Software/profile revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
