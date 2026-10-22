# Delivery roadmap

Status: implementation begun, 2026-10-22. Sprint 00's engineering foundation and Sprint 01 measurement ingestion are implemented; verification evidence and independent review are tracked in their [foundation report](SPRINT_00_IMPLEMENTATION.md) and [data report](SPRINT_01_IMPLEMENTATION.md). Sprints 02–11 remain **Not started**. No dataset has been approved for an independent benchmark. Independent completion sign-offs remain pending.

## Execution model

Execute numbered sprints in order for a single implementation stream. Owners are accountable roles, not assigned people; assign a named engineer and reviewer at sprint kickoff. A person may hold multiple engineering roles, but scientific claims require independent review. These sprints are bounded work packages, not promised calendar durations. Estimate effort after the Sprint 00 dependency spikes and data inventory.

Do not mark work complete based on a checklist alone. Each acceptance test needs an immutable evidence artifact containing the command, inputs, software revision, expected result, actual result, and reviewer disposition. Dependencies may be developed against documented fixtures; dependent milestones cannot close before the actual dependency passes.

## Sprint index

| Sprint | Objective | Accountable owner | Hard dependencies | Exit deliverable |
| --- | --- | --- | --- | --- |
| [00](sprints/00-foundation.md) | Rust workspace, CLI, governance, CI | Platform lead | This directive and plan | Reproducible Rust build and protocol skeleton |
| [01](sprints/01-data-provenance.md) | Ingestion and provenance | Data engineering lead | 00 | Versioned assays, Parquet/Zarr, immutable ingestion |
| [02](sprints/02-ontology-profile.md) | Ontology and reference profile | Computational biology lead | 00, 01 | Typed cell/population state and illustrative WJ profile |
| [03](sprints/03-causal-graphs.md) | Graphs and identification | Causal inference lead | 00, 02 | Graph validation, d-separation, supported identification certificates |
| [04](sprints/04-scm-counterfactuals.md) | SCMs and three causal query levels | Causal inference lead | 01, 03 | Analytically verified interventions and counterfactuals |
| [05](sprints/05-dynamic-simulation.md) | Time-dependent numerical engine | Numerical methods lead | 02, 04 | Offline ODE simulation and replayable runs |
| [06](sprints/06-licensing-secretome.md) | Licensing and secretome modules | Computational biology lead | 01–05 | Complete illustrative terminal demonstration |
| [07](sprints/07-jev-routing.md) | Jev routing and calibration | AI integration lead | 00, 04, 06 | Typed advisory routing with deterministic fallback |
| [08](sprints/08-reasoning-models.md) | Deliberative models and tools | AI integration lead | 03, 06, 07 | Local reasoning adapter and isolated proposals |
| [09](sprints/09-fitting-uncertainty.md) | Fitting, uncertainty, hybrid models | Statistical modeling lead | 01, 04–06 | Fit artifacts, uncertainty reports, baseline comparisons |
| [10](sprints/10-independent-evaluation.md) | Independent biological benchmarks | Evaluation lead | 01, 06, 09; licensed eligible data | Locked evaluation and bounded claims report |
| [11](sprints/11-hardening-release.md) | Hardening, audit, release | Release lead | 00–10; see release tiers below | Audited research release with explicit evidence status |

## Milestones

| Gate | Sprints | Required demonstration | Permitted claim |
| --- | --- | --- | --- |
| M0: engineering foundation | 00 | Offline Rust binary, stable error envelope, CI evidence | Platform skeleton |
| M1: data and identity | 01–02 | Provenance-preserving imports and profile inspection | Typed representation with illustrative state |
| M2: mathematical engine | 03–04 | Three query levels and analytic fixtures, including refusals | Correctness on the tested mathematical model classes |
| M3: initial research instrument | 05–06 | Entire [terminal demonstration](DEMO.md), including manifest replay | Illustrative mechanistic simulation |
| M4: optional intelligence | 07–08 | Unavailable-service fallback; proposals cannot mutate models | Advisory AI integration |
| M5: fitted research models | 09 | Frozen fits, uncertainty, train-only baselines | Dataset-specific fitted behavior; no independent claim yet |
| M6: independent evaluation | 10 | Untouched holdouts, actual errors and calibration | Only claims supported within evaluated domains |
| M7: release | 11 | Reproduction, security, compatibility and evidence audit | Research release at its documented evidence tier |

M3 is intentionally usable before AI integration or fitting. Adding Jev or a reasoning model must not change a deterministic simulation for an otherwise identical manifest.

## Critical path and external dependencies

The engineering critical path is 00 → 01 → 02 → 03 → 04 → 05 → 06. Numerical work must not conceal missing causal semantics behind an AI-generated explanation.

Independent data acquisition begins in Sprint 01, including dataset eligibility, donor and intervention metadata, licensing and access. Sprint 10 depends on adequate experimental data, not just engineering completion. Do not fabricate accessions or substitute synthetic observations when data are unavailable.

Jev credentials are optional for live integration checks. All provider contract, timeout, malformed-response and fallback tests must be reproducible with Rust fixtures. Local reasoning needs compatible licensed weights and a verified Rust backend; if absent, keep the adapter unavailable and do not claim local inference has passed.

## Execution procedure

1. Confirm entry criteria and assign named owners. Record unresolved scientific or dependency questions.
2. Implement the sprint's tasks in listed order, using stable task IDs in commits and evidence.
3. Run its acceptance suite and shared contract checks. Record failures as failures.
4. Update evidence links, limitations and remaining risks. An unchecked mandatory item prevents completion.
5. Obtain engineering and applicable scientific sign-off; record date and software/model revisions.
6. Move to the next sprint. Reopen affected acceptance tests whenever a later change invalidates their assumptions.

## Release tiers and deferred scope

An **illustrative preview** may be packaged after M3 and relevant hardening work, clearly stating which later sprints remain incomplete. This is not full Sprint 11 or project completion. A **complete research platform release** requires all mandatory sprint gates, including execution of independent benchmarks; a negative benchmark can be a valid completed evaluation. A **supported prediction claim** additionally requires positive evidence under predeclared criteria. If eligible data are missing, Sprint 10 and full project completion remain blocked.

Deferred unless introduced through a separate scoped decision: complete biochemical reconstruction; clinical decision support; arbitrary cyclic equilibrium SCMs; full general-purpose causal identification; arbitrary nonlinear counterfactual inference; general SDE/pathwise counterfactuals; autonomous wet-lab control; and comprehensive differentiation dynamics. Unsupported requests receive explicit status, never placeholder numbers.
