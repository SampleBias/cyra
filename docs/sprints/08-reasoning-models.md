# Sprint 08 — Deliberative reasoning and scientific tool protocols

Status: **Not started**. Accountable owner: **AI integration lead** (named assignee pending). Reviewers: platform/security engineer and causal/scientific reviewer. Milestone: **M4**, completion.

## Objective

Add a separate reasoning interface for literature-supported hypotheses, graph review, interpretation and candidate comparisons. Provide a tested local Rust inference path; keep all outputs advisory and traceable.

## Dependencies and entry criteria

- Sprints 03, 06 and 07 accepted; approved-model immutability, scientific tools and optional routing are established.
- Identify legally usable local weights and tokenizer with a compatible Rust backend. Candidate [Candle documentation](https://docs.rs/candle-core/latest/candle_core/) is a starting point, not proof of model compatibility.
- Model/backend/license compatibility must be demonstrated before claiming local reasoning support.

## Actionable backlog

- [ ] **S08-01 — ReasoningProvider contract** (AI): Define separate typed operations/results for hypothesis formation, graph critique, simulator interpretation and proposed experimental comparisons. Include evidence references, uncertainty/limitations, model/prompt hashes and malformed-output errors.
- [ ] **S08-02 — Local inference spike** (AI): Select a specific compatible reasoning-capable model after checking architecture, tokenizer, weights, quantization, license, context/memory and Rust backend support. Demonstrate CPU inference without Python/JavaScript or a non-Rust inference engine. Record alternatives and measured limitations in an ADR.
- [ ] **S08-03 — Local adapter** (AI): Implement bounded loading/inference, cancellation, context limits and controlled decoding. Record weight/tokenizer hashes, backend/features, prompt template, generation settings and seed. Missing weights return unavailable; never auto-download under `--offline`.
- [ ] **S08-04 — Optional hosted adapter** (AI): If included, isolate Rust HTTPS configuration and opt-in payload handling from the local/default runtime. Use independent feature flags, deadlines and redaction. Hosted execution is an optional scope item; omission does not block this sprint.
- [ ] **S08-05 — Literature evidence bundles** (Biology + AI): Supply curated, source-addressable excerpts/records with retrieval dates, supported claims, context and rights. Require citations to resolve to supplied source IDs; unknown citations stay unverified and cannot justify approval.
- [ ] **S08-06 — Scientific tool interface** (Platform): Expose typed read-model, inspect-provenance, graph-check, query-result and budgeted simulation tools. Enforce artifact allowlists, request schemas, work limits and immutable outputs. No arbitrary shell, unrestricted filesystem, model approval or measurement writes.
- [ ] **S08-07 — Proposal lifecycle** (Causal + Platform): Implement immutable graph amendment proposals with base revision, edge/equation changes, expected consequences, sources, conflicts and suggested validation. Route through deterministic validation and explicit human review before producing a new approved revision.
- [ ] **S08-08 — Candidate comparisons** (Biology): Generate structured hypothesis/estimand, candidate interventions, outcomes, measurement requirements, identification assumptions and feasibility limits. Mark suggestions unexecuted; computational comparisons and experimental evidence remain distinct.
- [ ] **S08-09 — Interpretation boundary** (AI): Attach optional attributed interpretation to deterministic `explain` output without altering the recorded numerical result. Reject invented measurements, unresolved data citations and unsupported numerical claims as unverified content.
- [ ] **S08-10 — Evaluation and CLI** (Evaluation + Platform): Implement `reason` and proposal/review commands, JSONL equivalents, and a fixed evidence-grounded evaluation suite. Report citation validity, unsupported claims, protocol compliance, reviewer agreement, latency and offline behavior.

## Deliverables

Reasoning interface, tested local Rust adapter, model/backend compatibility ADR, curated evidence fixtures, bounded tool protocol, proposal/review lifecycle and advisory-result evaluation report. Hosted adapter is optional and separately reported.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A08-01 | Load pinned licensed weights and perform local CPU inference | Actual successful inference with hashes/backend report, no external service or interpreter |
| A08-02 | Weights absent/corrupt or reasoning adapter unavailable | Clear optional-feature status; deterministic simulation/explanation still succeeds |
| A08-03 | Generate graph amendment then inspect approved model | Proposal isolated; approved revision hash unchanged |
| A08-04 | Review and approve a valid proposal through authorized CLI workflow | New model revision and review record; prior experiments preserve old model |
| A08-05 | Malicious source text requests shell execution or measurement fabrication | No forbidden tool dispatch or mutation; output flagged/rejected under deterministic rules |
| A08-06 | Proposal cites absent/contradictory source and invents a measurement | Citation remains unverified; measurement store unchanged; proposal cannot be auto-approved |
| A08-07 | Tool request exceeds time/memory/artifact allowlist | Bounded failure with no escaped access or uncontrolled execution |
| A08-08 | Interpret known analytic result and compare to deterministic explanation | Numeric result unchanged; sourced interpretation and limitations distinguishable |
| A08-09 | Run fixed evidence-grounded review/hypothesis tasks | Actual quality, unsupported-claim and latency report; no assumed correctness from valid JSON |

## Evidence to collect

Store `evidence/sprint-08/<run-id>/` with compatibility ADR, model/license hashes, local inference transcript, tool access tests, proposal diffs/review records, citation checks and evaluation report. Keep hosted evidence separate if that option is implemented. Current evidence: **none**.

## Limitations and risks

Rust backends may not support chosen model architectures or quantizations; select based on a working spike. Local generation may not be bitwise portable; record the actual reproducibility tier. Evidence-grounded generation can still be wrong. A proposal is a research hypothesis, never an approved causal mechanism or new experimental observation. Missing weights block local-inference acceptance, not the scientific runtime.

## Completion sign-off

- [ ] Mandatory S08 tasks and A08-01 through A08-09 completed; optional hosted scope explicitly reported.
- [ ] Actual local inference verified and model/tool permission boundary audited.
- [ ] Scientific reviewer approved proposal and citation review requirements.

Engineering approver: **pending**. Causal/scientific reviewer: **pending**. Software/model/backend revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
