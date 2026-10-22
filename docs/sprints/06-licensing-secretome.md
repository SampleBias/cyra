# Sprint 06 — Inflammatory licensing and secretome modules

Status: **Not started**. Accountable owner: **Computational biology lead** (named assignee pending). Reviewers: MSC domain scientist, numerical engineer and causal reviewer. Milestone: **M3**, completion.

## Objective

Implement the first coherent illustrative WJ-MSC mechanism bundle, prioritizing licensing and secretion, then oxidative stress, viability, proliferation and senescence. Deliver the complete offline terminal demonstration.

## Dependencies and entry criteria

- Sprints 01–05 accepted; provenance, ontology, causal semantics and dynamics are available.
- Literature curation starts from [BIO-02 and BIO-03](../REFERENCES.md); numeric values require exact source extraction or an illustrative label.
- Functional assays and molecular outputs have separate observation types.

## Actionable backlog

- [ ] **S06-01 — Mechanism evidence dossier** (Biology): Curate candidate inflammatory pathways and secretion mechanisms with tissue, species, culture, passage, stimulus, assay and time context. Separate directional biological hypotheses from evidence for specific equations/rates. Record conflicting and missing evidence.
- [ ] **S06-02 — Licensing module** (Biology + Numerical): Implement a bounded activation/decay proxy first. Document how external inflammatory exposure drives it; avoid labeling a latent activation fraction as measured pathway activity. Add named molecular nodes only where their interpretation is defensible.
- [ ] **S06-03 — Secretome mass balance** (Biology): Implement per-cell production, intracellular versus extracellular compartments, medium volume, live-cell count, clearance and collection/exchange events. For analyte j use an explicit balance such as `dCj/dt = Nlive*qj/V - kj*Cj` under constant V, and separately define dilution/jump events.
- [ ] **S06-04 — Analyte/functional boundaries** (Biology): Define a minimal source-reviewed panel or explicitly generic illustrative analyte. Keep transcript expression, intracellular enzyme abundance/activity, secreted protein and functional immune assay distinct. Do not treat intracellular IDO-related activity or a nonprotein mediator as a secreted protein assay.
- [ ] **S06-05 — Oxidative stress and viability** (Biology + Numerical): Add documented illustrative stress production/removal and live/dead transition mechanisms with nonnegative rates. Track viable fraction denominators and distinguish secretion from analyte release by cell lysis.
- [ ] **S06-06 — Proliferation and senescence** (Biology): Add a minimal bounded population model with proliferating/senescent/dead compartments or equally explicit states. Define conservation/transfers and growth/death terms; label passage effects and senescence irreversibility as assumptions unless supported.
- [ ] **S06-07 — Parameter registry audit** (Biology): For every value include unit, domain, origin, source locator or illustrative rationale, uncertainty, applicable context and fit status. Literature-motivated mechanism forms do not make arbitrary numeric constants literature-derived.
- [ ] **S06-08 — Scenarios and graph integration** (Causal + Biology): Add baseline, licensing pulse, oxidative challenge and medium-exchange simulation scenarios. Ensure state transitions and observation operators align with the approved illustrative graph; distinguish input interventions from direct state clamps.
- [ ] **S06-09 — Complete terminal demo** (Platform): Materialize all files and implement the [documented sequence](../DEMO.md). Show dynamics, structural intervention, supported toy counterfactual, deterministic explanation and manifest replay from one terminal. Also run the sequence through JSONL.
- [ ] **S06-10 — Model card and research preview** (Biology + Release): Document exact supported states/time ranges, illustrative assumptions, unsupported physiology and mathematical versus empirical evidence. Produce an inspectable M3 artifact bundle; no physiological validation claim.

## Deliverables

Illustrative WJ mechanism bundle, pathway/equation dossier, parameter registry, source/assumption graph, scenario files, model card, deterministic explanations and a fully reproduced terminal demonstration.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A06-01 | `dL/dt=a(1-L)-bL` with demo constants and L(0)=0 | Matches `(2/3)(1-exp(-1.5t))` within declared tolerance |
| A06-02 | Constant secretion/clearance, zero cells, doubled volume and medium exchange | Correct mass balance, dimensions, limiting behavior and event jumps |
| A06-03 | Population compartment transitions with growth/death disabled | Mass/count conservation where assumed; nonnegative states |
| A06-04 | Stress or viability boundary cases | Explicit model-supported behavior; no NaNs, hidden negative counts or unreported clipping |
| A06-05 | Query protein secretion from transcript-only data or potency from concentration alone | Unsupported inference without an approved connecting mechanism/assay model |
| A06-06 | Audit all biological parameters and model edges | Every value sourced or illustrative; every assumption/context visible |
| A06-07 | Execute every command in the terminal demo without network/credentials | Correct analytic intervention/counterfactual, time trajectories, explanation and replay |
| A06-08 | Execute equivalent JSONL requests and inspect metadata | Same scientific results; no terminal decorations/logs in stdout |
| A06-09 | Review report wording and unsupported differentiation/metabolism queries | Illustrative/synthetic claims only; unmodeled behavior explicitly unavailable |

## Evidence to collect

Store `evidence/sprint-06/<run-id>/` with source reviews, parameter audit, analytic/error reports, population balance checks, both demo transcripts, run/replay manifests and model card. Record the actual comparison to each analytic expectation. Current evidence: **none**.

## Limitations and risks

Mechanisms are simplified and parameters initially illustrative. Source tissue, donor, passage, assay and exposure history may alter responses; no universal secretome or potency relationship is assumed. Functional immune effects may require cell contact or additional context beyond soluble measurements. Differentiation and detailed metabolism remain represented but not fully simulated. Supported counterfactual mathematics in the demo does not establish dynamic physiological abduction.

## Completion sign-off

- [ ] S06-01 through S06-10 completed; A06-01 through A06-09 passed.
- [ ] MSC reviewer approved labels, compartment boundaries and stated limitations.
- [ ] M3 demo is reproducible from manifests on the reference runtime.

Engineering approver: **pending**. MSC/causal reviewer: **pending**. Software/profile/model revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
