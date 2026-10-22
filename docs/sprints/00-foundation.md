# Sprint 00 — Rust workspace, CLI, governance and CI

Status: **In progress — engineering implemented; independent sign-off pending**. Accountable implementation owner: **Codex (platform implementation agent)**. Independent reviewers: pending assignment. Milestone: **M0**. See the [implementation report](../SPRINT_00_IMPLEMENTATION.md).

## Objective

Establish a reproducible Rust-only foundation and stable headless interfaces before introducing biological or causal calculations. This sprint produces working infrastructure, not a cell simulator.

## Dependencies and entry criteria

- Governing directive and [engineering specification](../SPECIFICATION.md) reviewed.
- Named owner assigned; repository license decision and supported reference target recorded.
- Rust toolchain availability checked. Network access may be needed to acquire dependencies once; scientific execution must not require it.
- No previous sprint dependency.

## Actionable backlog

- [x] **S00-01 — Workspace and toolchain** (Platform): Create `Cargo.toml`, `rust-toolchain.toml`, `Cargo.lock`, `cyra-core` and `vmcell`. Pin the tested toolchain/MSRV policy and document CPU reference target. Produce a release-mode binary without scripting runtimes.
- [x] **S00-02 — Dependency feasibility record** (Platform): Inspect candidate serialization, CLI, hashing, units, RNG, storage, linear algebra and ODE dependencies. Record versions/licenses/features/native dependencies. Prove a minimal Rust-only path; defer unnecessary crates. Capture decisions in `docs/adr/0001-runtime-boundary.md`.
- [x] **S00-03 — Scientific result/error types** (Platform + Causal): Implement IDs, version tags, status/error enums, provenance references and scientific context. Serialize a typed rejection without fabricating a result. Reserve distinct association/intervention/counterfactual request types.
- [x] **S00-04 — CLI skeleton** (Platform): Implement `version`, `capabilities`, `schema` and help. Register planned commands as explicitly unavailable or omit them until implemented; no success stubs. Add human and JSON modes with stderr diagnostics.
- [x] **S00-05 — JSONL dispatcher** (Platform): Implement the [protocol contract](../CLI_CONTRACT.md), request IDs, strict parsing, bounded lines/nesting, one response per line and recoverable error handling. Route to the same typed services as the CLI.
- [x] **S00-06 — Artifact primitives** (Platform): Implement canonical serialization rules, hashing, atomic file creation, no-overwrite writes and private directory/file creation. Separate stable scientific artifacts from execution timestamps. Include interruption cleanup.
- [x] **S00-07 — Governance records** (Scientific lead): Add ADR, model card, evidence-manifest and review templates from [governance](../GOVERNANCE.md). Distinguish illustrative, measured, fitted and independently evaluated statuses in types.
- [x] **S00-08 — Continuous integration** (Release): Add formatting, linting, tests, release compilation and dependency/license checks. Pin CI tooling/actions, use least-privilege job permissions, and test a prepared offline dependency cache. All functional harnesses use Rust; shell may orchestrate binaries.
- [x] **S00-09 — Platform and performance contract** (Platform): Record CPU/OS, reference release workload policy, reproducibility tier, initial operational budgets and how to report latency/memory. Treat budgets as engineering targets pending measurement.
- [x] **S00-10 — Contributor instructions** (Platform): Document build, test, protocol usage, artifact rules, review expectations and how to attach evidence to a sprint. Make a fresh-checkout build reproducible without undocumented local configuration.

## Deliverables

Rust workspace, binary skeleton, protocol/result schemas v1, CI configuration, runtime-boundary ADR, governance templates and a first execution-evidence manifest. Future crate names do not require empty placeholder implementations.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A00-01 | Build and run release binary in a Rust build environment without Python/Node executables | `version` and `capabilities` succeed; runtime invokes no interpreter |
| A00-02 | Disable network after dependency acquisition and run core tests/binary | Deterministic functionality succeeds without credentials or external requests |
| A00-03 | Send valid, malformed and valid JSONL lines | Exactly three valid response objects; request failure does not corrupt the following request |
| A00-04 | Unknown operation/version, duplicate keys/IDs, overlong/deep payload | Stable error codes; bounded memory; no panic or partial mutation |
| A00-05 | Capture stdout and stderr with/without a TTY | Structured stdout has only parseable JSON; no ANSI, banners or logs |
| A00-06 | Write an artifact twice, interrupt a write, and use a malicious referenced path | No overwrite, no published partial result, no bundle path escape |
| A00-07 | Inspect permissions and scan error/log output for fixture secrets | Owner-only defaults on supported Unix target; no secret leakage |
| A00-08 | Fresh checkout and pinned CI checks | `cargo fmt --check`, configured Clippy policy, `cargo test --workspace --locked`, and release build pass |

## Evidence to collect

Execution evidence: [foundation manifest](../../evidence/sprint-00/2026-10-22-foundation/manifest.json) and [environment/clean-source checks](../../evidence/sprint-00/2026-10-22-environment/REPORT.md). These record actual commands, outcomes, hashes and environment limits. Source and binary revisions are content hashes; no independent physiological validation is asserted.

## Limitations and risks

No biological behavior or causal effect is implemented here. Optional crates may bring native codecs or GPU toolchains; audit default features before adding them. A cached offline build is distinct from acquiring packages offline. Performance budgets are provisional until measured.

## Completion sign-off

- [ ] S00-01 through S00-10 completed; A00-01 through A00-08 passed.
- [ ] Shared definition of done satisfied; evidence links attached.
- [ ] Runtime boundary and scientific status types reviewed.

Implementation owner: **Codex**. Independent engineering approver: **pending**. Scientific reviewer: **pending**. Software version: **0.1.0**, with source-inventory hash in the evidence manifest. Implementation date: **2026-10-22**. Exit decision: **engineering evidence available; independent sign-off not recorded**.
