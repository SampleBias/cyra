# Cyra engineering specification

Version: implementation contract 0.1, 2026-10-22. Source authority: the user's original VMSC directive and explicit rename to Cyra — Cybernetic Cellular Intelligence. Sprint 00's foundation and Sprint 01 measurement ingestion are implemented. Profiles, causal graphs, simulation, fitting and evaluation remain planned. No physiological validation is implied.

## 1. Product and delivery scope

Build a headless research instrument, named Cyra with CLI `vmcell`, that represents human MSCs and computes explicit biological hypotheses. Start with a cultured human Wharton's jelly-derived MSC profile; support additional tissue profiles through versioned bundles. The default scientific runtime is Rust, CPU-capable, deterministic when configured, and independent of external AI services.

The initial deliverable uses illustrative biological parameters. It provides data/provenance inspection, valid causal queries for specified model classes, time evolution, intervention schedules, explanations derived from actual computation, and reproduction from immutable manifests. Learned physiological prediction is a later, separately evaluated capability.

## 2. Proposed Rust workspace

| Crate | Responsibility | Permitted dependencies |
| --- | --- | --- |
| `cyra-core` | IDs, units, domains, schemas, provenance and error types | Rust utility crates; no network or AI |
| `cyra-data` | Assays, imports, Parquet/Zarr, manifests and split definitions | Core and storage libraries |
| `cyra-bio` | Cell/population ontology, profile bundles and observation operators | Core; no inference service |
| `cyra-causal` | Graphs, identification, structural equations, inference and counterfactuals | Core and numerical libraries |
| `cyra-sim` | ODE evolution, events, ensemble execution and constraints | Core, bio, causal |
| `cyra-eval` | Fitting, baselines, uncertainty and holdout evaluation | Core, data, causal, sim |
| `cyra-ai` | Typed advisory interfaces, Jev, optional local/hosted reasoning | Core and explicitly enabled adapters |
| `vmcell` | CLI, JSONL dispatcher, orchestration and artifact access | Public APIs of the above |

Create crates incrementally; do not add empty frameworks solely to match this table. Library APIs must be independently testable without CLI parsing. Scientific crates must not depend on AI adapters. No GUI, Python subprocess, JavaScript runtime, or mandatory remote database belongs in the default path.

Dependency selection is a Sprint 00/01 spike: pin the toolchain, MSRV policy and lockfile; inspect feature graphs and native code. Use Rust implementations for default numerical, storage and inference paths. Document normal operating-system linkage separately from scientific-library FFI. Optional GPU/native acceleration must be feature-gated and excluded from the default guarantee. Begin storage with uncompressed codecs if needed to avoid a hidden native build dependency.

## 3. Identity, state and observation contracts

Use distinct `CellState`, `PopulationState`, `CultureContext`, `MeasurementRecord` and `ProfileBundle` types. A cell has a stable cell ID and donor/batch links. A population has membership or distribution/aggregation semantics, counts, sampling frame, and shared culture context. A bulk assay is a population measurement and cannot be silently copied into every cell.

Represent these domains explicitly:

| Domain | Required representation and boundaries |
| --- | --- |
| Identity | Species, tissue source, culture history, characterization evidence, marker assay context |
| Expression | Gene IDs/namespace, transcript modality, units/normalization and observation time |
| Intracellular proteins | Protein IDs, abundance/activity distinction, compartment and assay |
| Secretion | Per-cell flux, extracellular amount/concentration, collection duration and volume as distinct quantities |
| Metabolism | Named metabolites/fluxes or explicitly illustrative latent proxies; no undocumented scalar “health” |
| Viability | Live/dead state or counts and assay observation model, with denominator |
| Proliferation | Division dynamics/rates and separate measured proliferation readouts |
| Senescence | Latent state/proxy and assay evidence kept separate; irreversibility only if assumed |
| Oxidative stress | Named measurements or bounded illustrative state with units and interpretation |
| Licensing | Exposure history, time-dependent signaling states and conditions |
| Differentiation | Lineage-specific assay evidence and potential/status; unmodeled kinetics remain unsupported |
| Culture | Passage, duration, medium/batch, supplements, oxygen, temperature, density, vessel and volume when available |

Measurement metadata: donor pseudonym, cell/population/sample/replicate IDs, tissue, passage, culture batch, experimental batch, assay type/protocol version, compartment, analyte ID, time and time origin, value/units, uncertainty or detection limit, source accession/version, file hash, provenance chain and data rights.

Missingness is typed: `not_measured`, `below_detection`, `failed_qc`, `not_applicable`, `unknown`. Below detection is censoring with a limit, not zero. Derived/imputed values occupy separate artifacts with method provenance; original measurements remain missing. Profile defaults are parameter assumptions, never experimental observations.

## 4. Scientific storage and manifests

Use versioned JSON metadata/manifests, Parquet measurement tables and Zarr omics/time-series arrays. Sprint 01 commits `cyra.parquet.v1` and `cyra.zarr.v1`: uncompressed codecs, canonical decimal strings, bounded row groups and chunks, and missing chunks as errors rather than zeros. See [ADR 0003](adr/0003-storage-profiles.md). Large arrays must support bounded-memory slicing. Import explicit CSV/TSV/JSON mappings first; advanced assay-format adapters are opt-in additions, not silent guesses.

Ingestion is staged: validate → hash → normalize → validate derived artifacts → atomically publish an immutable manifest. Failures leave no apparently successful dataset. Preserve original units and transformations. Never merge donor identities by name similarity. Source accessions and licenses must resolve before data enter a publishable/evaluation-eligible dataset.

An experiment manifest contains schema version; command/typed request; profile, graph, equations and parameter revisions/hashes; input datasets; source/license references; solver and tolerances; intervention schedules; evidence conditioned on; RNG algorithm and seed/substreams; toolchain/dependency/build fingerprints; architecture/thread policy; uncertainty settings; expected artifact hashes or numerical tolerances; and restrictions on interpretation.

Avoid a hash cycle: freeze an input manifest before execution, then create an immutable result manifest referencing its hash and output artifacts. A volatile execution record carries wall-clock time and duration separately. Replay creates a new execution record while preserving the scientific input identity. File permissions alone are not immutability: use content hashes, atomic creation and overwrite refusal.

## 5. Causal model contract

Adopt structural models `M = (U, V, F, P(U))`. Every endogenous variable has a typed domain, units, equation and explicit parents. Exogenous variables have named distributions/dependence and provenance. Equations are a restricted, validated Rust expression representation or registered Rust mechanisms, never arbitrary evaluated scripts.

Keep measured observation nodes distinct from latent biological state. Measurement error has its own mechanism; conditioning on a noisy assay does not set the latent state equal to its readout. Shared donor or culture variation is an explicit modeled source of dependence, not an inference from correlation alone.

Support acyclic static SCMs and finite time-unrolled transition graphs initially. Within a slice reject directed cycles; feedback is represented through time. Validate equation/graph parent agreement, domains, units, exogenous dependencies and missing mechanisms. ODE dependencies describe transitions rather than invalidating the acyclic query model. Do not assume an arbitrary continuous-time model has a solved counterfactual posterior.

### Query levels

| Type | Mathematical target | Required behavior |
| --- | --- | --- |
| Association | `P(Y | X=x, E=e)` or a declared descriptive statistic | Condition/filter observations or a specified joint model; no graph surgery |
| Intervention | `P(Y | do(X=x))` or a declared contrast | Replace targeted structural mechanisms; remove incoming edges; preserve remaining mechanisms |
| Counterfactual | `P(Y_x | E=e)` | Infer `P(U | E=e)`, apply action, predict using the same inferred exogenous background |

Requests must specify the target population, outcome, intervention/value or contrast, time horizon, conditioning evidence and inference basis. Never change the query level to return an answer. Return unit-level and population-level results under distinct types.

### Identification versus model evaluation

Track two separate axes:

1. `identification`: `identified`, `not_identifiable`, `unsupported`, or `not_applicable`, with graph revision, assumptions, available data and certificate/reason.
2. `computation`: `observational_estimate`, `identified_effect_estimate`, `specified_scm_prediction`, or `counterfactual_under_scm`.

A fully specified illustrative SCM can yield predictions conditional on its structural assumptions while its effect is not identifiable from available observational data. Require `basis=specified_scm` for that computation; never present it as an observationally identified effect. A rejected observational-identification query must not silently fall back to model assumptions.

Initial graph algorithms: topological validation, ancestors/descendants, d-separation on DAGs with explicit latent nodes, modeled common-cause/back-door path reporting, and applicability checks for back-door adjustment and the standard single-treatment/single-mediator front-door criterion. Candidate adjustment sets must use available measured variables. Return a symbolic estimand/certificate when supported, plus positivity requirements before estimation.

Limit adjustment-set enumeration by a declared budget. Exhausting a search budget or lacking a general identification algorithm returns `unsupported` or `resource_limit`; it is not proof of non-identifiability. Recognized non-identifiable cases require a defensible witness or supported criterion. No hidden-confounder discovery from correlations is promised. ADMG projection/m-separation and general identification calculus are later extensions unless explicitly implemented and tested.

### Initial inference families

- Finite discrete acyclic SCMs: exact enumeration and conditional inference within resource bounds.
- Linear additive Gaussian SCMs: analytical conditioning and affine interventions with validated covariance handling.
- Registered deterministic nonlinear mechanisms: forward simulation; counterfactual queries only where an explicitly implemented abduction method applies.

Reject impossible discrete evidence; use regular conditional Gaussian inference for continuous evidence rather than testing an exact continuous event for nonzero probability. Diagnose singular/inconsistent evidence separately. Partial evidence yields a posterior distribution, not an arbitrarily chosen residual. Deterministic zero residual uncertainty is labeled conditional on fixed parameters; it does not imply physiological certainty.

## 6. Mechanistic dynamics

Evolve `dx/dt = f(x, c(t), theta)` with typed states, culture forcing and event schedules. Implement a deterministic fixed-step reference integrator and an adaptive solver with error control; validate both against closed-form solutions. Select or implement the solver in Rust after license, accuracy and feature review. Store absolute/relative tolerances per state scale, maximum steps, event order and termination reasons.

Prioritize a licensing activation/decay module and secretion mass balance. Add oxidative stress, viability, proliferation and senescence as explicitly bounded modules. For each mechanism provide the equation, units, initial conditions, parameter domains, provenance, invariant constraints, applicable culture/time range and unsupported regimes.

Extracellular concentration is related to production via cell number and volume; it also depends on clearance, dilution, collection and medium exchange. Population-derived secretion must not double-count cell number. Distinguish active secretion, cell lysis and measured total analyte. Functional potency requires a separately supported assay model.

Represent events as `do` clamps, pulse inputs, mechanism replacements or culture operations with precise duration and release behavior. A change in dose is not automatically a direct intervention on intracellular state. Enforce bounds using justified methods and adaptive rejection, not silent clipping. Report solver failure and preserve only explicitly labeled partial output.

Stochastic support begins with documented ensembles over initial states, parameters or exogenous variables. This is not an SDE implementation. Split random streams by stable identities; record aggregation and coupling. Counterfactual trajectories share inferred backgrounds and appropriately coupled noise only when that dynamic abduction model is supported; otherwise return `unsupported`.

## 7. Jev decision layer

Use typed `Choice<Route>`, ordered `Score<Rubric>` and `Noul` wrappers whose domains are validated in Rust. Applications: query classification, route suggestions, anomaly triage, uncertain-context detection and escalation. Primitive meanings follow the [official TypeSafe documentation](https://docs.typesafe.ai/primitives); Noul returns a yes-probability, not an uncertainty object.

Explicit CLI/JSONL requests take precedence over suggested routes. A versioned Rust policy determines thresholds, allowed actions and escalation. No AI output can override scientific validation, authorize model mutation or declare identifiability. Keep routing confidence distinct from scientific uncertainty.

Default fallback: dispatch valid typed requests directly; use deterministic input/schema/constraint checks for anomalies; return `needs_explicit_query` for ambiguous free text; report provider unavailability and policy revision without inventing probabilities. Missing credentials, timeout, rate limit, malformed data and service outage must leave offline simulation usable. Hosted access is opt-in and not attempted under `--offline`.

## 8. Deliberative reasoning

Provide a separate `ReasoningProvider` trait for structured hypotheses, graph reviews, simulator interpretation and candidate experimental comparisons. Evaluate a compatible local model using a Rust inference backend first; pin weights, tokenizer, prompt template, model license and decoding settings. Compatibility is an acceptance test, not an assumption. Hosted Rust HTTPS adapters are optional and separately feature-gated.

Expose only bounded, typed, allowlisted tools for reading approved models/evidence and requesting simulations under a caller-supplied budget. The reasoning interface cannot write measurements, approve models or execute arbitrary shell commands. Candidate graph amendments are immutable proposals pending review. Treat retrieved literature and model text as untrusted content.

The default `explain` command is deterministic and generated from the actual estimand, structural equations, identification certificate, posterior/action trace and numerical result. Optional reasoning interpretation is an attributed supplement. Model prose cannot supply missing evidence or replace a failed computation.

## 9. Fitting and evaluation

Freeze dataset eligibility and splits before fitting. Group all cells, repeated samples and technical replicates from a donor together for donor generalization. Maintain distinct batch, passage and perturbation holdout suites; use crossed grouping where possible and disclose unavoidable confounding. Reserve an external study/donor set for independent evaluation. Test preprocessing, feature selection, imputation, calibration and hyperparameters never learn from held-out outcomes.

Baselines: train-only constant/mean predictor; regularized empirical regression; persistence/decay dynamics; and the simplest relevant mechanistic model. Compare every learned/hybrid model against relevant baselines on identical inputs, splits and measurement operators. More complexity is not a success criterion.

Report raw-unit MAE/RMSE and suitable scale-aware errors; categorical/probability Brier score or log loss; interval coverage and width; calibration curves; donor-level uncertainty; subgroup counts; latency/memory; numerical failures; provenance; and identification status. Use metrics appropriate to the measurement type and censoring. Missing ground truth makes a metric unavailable. Observational prediction accuracy cannot establish causal-effect accuracy.

Keep parameter, aleatoric, measurement, model-form and numerical uncertainty separate where quantified. An interval from parameter sampling alone is labeled accordingly. Fitting cannot turn an unidentifiable estimand into an identified one. Hybrid residual correction must not violate causal semantics or physical constraints; causal use requires an explicitly approved structural interpretation.

## 10. Interfaces, reproducibility and release

The [CLI contract](CLI_CONTRACT.md) defines commands, JSONL envelopes, limits and errors. Every operation is agent-accessible without a TTY. No terminal formatting or debug messages may appear in structured stdout. Artifacts are content-addressed; private files default to owner-only permissions where supported.

Reproducibility is bitwise on the pinned deterministic reference target when achievable, and tolerance-based across declared platforms/backends. Record RNG identity, ordering, thread policy and numerical environment; a seed alone is insufficient. Offline release tests use prebuilt artifacts or a prepared dependency cache, distinguishing package acquisition from scientific runtime access.

Release only with an evidence-based model card describing exact supported variables, tissues, interventions, assay contexts and time horizons. No validated MSC physiological prediction claim is permitted without independent experimental support. See [governance](GOVERNANCE.md) and [roadmap release tiers](ROADMAP.md) for handling blocked data gates honestly.
