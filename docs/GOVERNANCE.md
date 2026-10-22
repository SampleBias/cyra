# Scientific governance and completion rules

## Authority and responsibility

The user's directive is the governing requirement. [The specification](SPECIFICATION.md) and numbered sprints translate it into a reviewable implementation contract. Record changes in `docs/adr/NNNN-title.md` during implementation with motivation, alternatives, scientific consequences, affected schemas/tests, owner and approval. No ADR can waive a non-negotiable requirement.

Engineering owner: responsible for implementation and reproducibility. Scientific reviewer: responsible for biological interpretation and admissible claims. Causal reviewer: responsible for estimands, identification and counterfactual semantics. Data steward: responsible for eligibility, provenance, licensing and donor grouping. Assign actual people at kickoff; this plan does not invent approvals.

## Evidence classes

Every model, parameter, dataset and result carries separate evidence classifications:

| Field | Allowed meaning |
| --- | --- |
| `parameter_origin` | `illustrative`, `literature_derived`, `fitted`, or `fixed_mathematical_constant` |
| `measurement_origin` | `experimental`, `synthetic_fixture`, or `simulated`; never silently interchangeable |
| `validation_tier` | `unvalidated`, `synthetic_verified`, `internally_evaluated`, or `independently_evaluated` |
| `claim_status` | `illustrative_only`, `unsupported`, `supported_within_scope`, or `inconclusive` |

Independent evaluation does not imply success or universal validity. Literature-derived values require a source locator, population/context, units, extraction/transformation method and uncertainty. A citation about a mechanism is not a source for a numeric rate. An illustrative number remains illustrative even when the mechanism is plausible.

## Model approval workflow

`draft → reviewed → approved → retired`, with rejected proposals retained as records. Approved model bundles are immutable and content-addressed. A change creates a new revision linked to its parent; existing experiments retain their original bundle. Approval means permitted for its stated research scope, not validated physiology.

A graph amendment proposal contains the base revision, added/removed edges, equations affected, supporting and conflicting evidence, identification consequences, suggested comparisons and author/provider provenance. Deterministic validation and a recorded human review are required before a new revision is approved. Reasoning tools and Jev have no model-approval or measurement-write capability.

## Data and licensing

Retain original sources and checksums; keep normalized and derived artifacts separate. Record accessions including dataset version, donor pseudonyms, batch, passage, assay conditions and exact transformations. Unknown donor identity prevents claims of donor-independent evaluation. Unknown or incompatible data rights place the dataset in a quarantined, evaluation-ineligible state until resolved; do not assume public accessibility grants reuse rights.

The original Cyra source is licensed under MIT. That license does not cover datasets, model weights, or third-party dependencies. Dataset permissions, software licenses and model-weight licenses are separate records. No raw donor identifiers or credentials belong in committed fixtures, logs or published reports. Remote AI transmission is opt-in and uses a minimized, recorded payload.

## Scientific claim rules

- Observed associations are reported as associations, even if predictive error is low.
- An effect derived from a completely specified SCM is conditional on its assumptions. It is not automatically identified from observational measurements.
- A mathematically non-identifiable query returns no point estimate. An unimplemented identification procedure returns `unsupported`, not a false proof of non-identifiability.
- A failed positivity/overlap check yields an estimation failure or bounded domain, even if a symbolic estimand is identified.
- Counterfactuals preserve the inferred exogenous background. Reusing a random seed alone is not abduction.
- Synthetic test success is never described as independent MSC physiology validation.
- Negative and inconclusive experimental results are preserved, including baseline wins and subgroup failures.

## Shared definition of done

- [ ] Every mandatory task and acceptance test in the sprint is complete.
- [ ] Tests exercise scientific meaning, invalid inputs and failure behavior, not just serialization snapshots.
- [ ] Model/schema/interface revisions and migration effects are documented.
- [ ] Scientific outputs include provenance, assumptions, uncertainty and identification status.
- [ ] Default runtime remains Rust-only and works without network credentials.
- [ ] Evidence manifest records commands, input/output hashes, toolchain, platform, seeds, timings and results.
- [ ] Limitations and unsupported cases are stated in user-facing contracts.
- [ ] Engineering and required scientific reviewers sign with real identities and dates.

Store sprint evidence under `evidence/sprint-NN/<run-id>/` when it exists. Minimum files: an immutable evidence manifest, structured test results, relevant reports and sign-off record. Exclude credentials and restricted raw data; use access-controlled references where needed. This planning delivery contains no such execution evidence.

## Completion states

Use `Not started`, `In progress`, `Blocked`, or `Complete`. A blocked live integration can coexist with a passing fallback, but cannot be labeled a completed live integration. Missing independent data block the corresponding biological acceptance gate. An illustrative preview may be released without claiming the full roadmap is complete.
