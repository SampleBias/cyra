# Sprint 11 — Hardening, documentation, audit and research release

Status: **Not started**. Accountable owner: **Release lead** (named assignee pending). Reviewers: platform/security engineer, causal reviewer, data steward and scientific lead. Milestone: **M7**.

## Objective

Release an audited, agent-usable Rust research instrument with reproducible artifacts, clear capability limits and evidence-linked claims. Release packaging cannot substitute for missing scientific acceptance.

## Dependencies and entry criteria

- All mandatory gates from Sprints 00–10 complete for full roadmap completion.
- Relevant hardening can begin earlier. An illustrative preview after M3 must identify incomplete sprints and is not full Sprint 11 completion.
- Resolve release license, supported platforms, model/data redistribution rights and named release signatories.

## Actionable backlog

- [ ] **S11-01 — Capability audit** (Release): Map every required command and scientific operation to supported model families, inputs, errors and tests. Remove or clearly reject unfinished stubs. Reconcile [acceptance traceability](../TRACEABILITY.md) with actual evidence.
- [ ] **S11-02 — Agent protocol hardening** (Platform): Fuzz parsing, schemas, operation dispatch and untrusted model/equation input using Rust tooling. Verify malformed-line recovery, EOF/cancellation, quotas, broken pipes, consistent errors and stdout purity.
- [ ] **S11-03 — Filesystem/security audit** (Security): Test traversal, symlinks, artifact replacement, temporary-file races, malformed archives, oversized allocations and hostile literature/tool payloads. Review credential redaction, private defaults and remote opt-in boundaries.
- [ ] **S11-04 — Numerical/scientific regression audit** (Causal + Numerical): Rerun accepted analytic, graph, abduction, solver, unit and baseline suites. Verify unknown/unsupported/non-identifiable statuses are retained; examine failed simulations and uncertainty exclusions.
- [ ] **S11-05 — Compatibility and migration** (Platform): Test supported schema readers/migrations, old manifests, CLI/JSONL clients and capability discovery. Document a versioning/deprecation policy and refuse incompatible artifacts with actionable errors.
- [ ] **S11-06 — Offline and feature matrix** (Release): Test default minimal Rust runtime, no-network/no-credentials execution, each optional adapter disabled/unavailable, and supported optional feature combinations. Audit native dependencies and keep optional acceleration outside default guarantees.
- [ ] **S11-07 — Performance and resource budgets** (Numerical + Release): Run frozen reference workloads, report p50/p95 latency, memory, model sizes, I/O and failures with hardware/build context. Resolve regressions against declared budgets; do not advertise unmeasured speedups.
- [ ] **S11-08 — Release packaging** (Release): Produce reproducible binaries/source bundles, checksums, dependency inventory/SBOM, license notices and provenance. Document dependency-cache preparation versus runtime offline operation. Publish nothing externally without the appropriate release authorization.
- [ ] **S11-09 — User and operator documentation** (Platform + Biology): Provide installation, every CLI/JSONL operation, profile/data authoring, model governance, error recovery, provenance/replay and the complete demo. Include a scientific limitations guide and example rejected queries.
- [ ] **S11-10 — Scientific claims audit** (Scientific lead): Check model cards and release notes against Sprint 10 claims. Distinguish mathematical verification, fitted behavior and independent biological support. If benchmarks fail, release honestly with those failures and no unsupported prediction claims.
- [ ] **S11-11 — Reproduction audit and handover** (Independent reviewer): Rebuild from a clean environment, reproduce the demo and selected evaluation manifests, verify hashes/tolerances, and record deviations. Assign maintainers and document incident/correction procedures for scientific and software errors.
- [ ] **S11-12 — Final release decision** (Release + Scientific lead): Record completed gates, open issues, supported domains, model/data licenses, software/model versions and actual signatories. An unresolved mandatory requirement prevents full-release sign-off.

## Deliverables

Audited release artifacts, schema/API reference, agent integration guide, model/data cards, dependency/license inventory, performance report, reproduction evidence, limitations guide and signed release decision.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A11-01 | Fresh clean build and complete demo in interpreter-free offline runtime | All default scientific functions work without Python, JavaScript or AI services |
| A11-02 | Exercise every required operation through both CLI and JSONL | Stable schema/exit behavior; no missing agent-only capability or polluted stdout |
| A11-03 | Parser/equation/import fuzz corpus and hostile artifact paths | Bounded failures; no panic-based success, arbitrary execution or unauthorized writes |
| A11-04 | Reproduce archived manifests with compatible versions | Hash/tolerance checks pass or explicit incompatibility; no silent changed science |
| A11-05 | Audit all causal success/refusal envelopes | Provenance, model revision, assumptions, identification and uncertainty retained |
| A11-06 | Disable/misconfigure every optional AI backend | Deterministic simulation and explanation remain usable |
| A11-07 | Review dependency/features, permissions, secrets and redistribution rights | Default Rust-only guarantee and documented rights supported by evidence |
| A11-08 | Run reference performance/large-input workloads | Resource budgets measured and met or release-blocking exceptions resolved |
| A11-09 | Independent demo and selected biological benchmark reproduction | Results consistent with frozen manifests and approved claims |
| A11-10 | Inspect release prose, model cards and unresolved mandatory work | No unsupported physiological claim; no full completion while required gates remain open |

## Evidence to collect

Store `evidence/sprint-11/<run-id>/` with feature/build matrices, protocol/fuzz/security results, dependency and license inventory, performance measurements, migration/replay reports, independent reproduction and signed release decision. Link Sprint 10's actual independent evidence rather than duplicating or relabeling synthetic results. Current evidence: **none**.

## Limitations and risks

The release remains a research instrument with a defined domain, not a universal virtual human cell. Unsupported nonlinear counterfactuals and unmodeled pathways must remain explicit. Cross-platform floating-point differences may require tolerance-based reproducibility. A preview can ship with incomplete later work only when labeled as such; it cannot satisfy the full directive by assertion.

## Completion sign-off

- [ ] S11-01 through S11-12 completed; A11-01 through A11-10 passed.
- [ ] Every non-negotiable requirement has linked evidence and a responsible reviewer.
- [ ] Full roadmap gates complete; release tier and permitted claims explicitly recorded.
- [ ] Release authorization and named signatories recorded before external publication.

Release approver: **pending**. Causal/scientific reviewers: **pending**. Data/security reviewers: **pending**. Release/software/model revisions: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
