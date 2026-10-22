# Sprint 07 — Jev typed routing and calibration

Status: **Not started**. Accountable owner: **AI integration lead** (named assignee pending). Reviewers: platform engineer and evaluation lead. Milestone: **M4**, first half.

## Objective

Integrate Jev as an optional TypeSafe System One advisory layer while preserving deterministic Rust control of query meaning, thresholds, authority and offline fallback.

## Dependencies and entry criteria

- Sprints 00, 04 and 06 accepted; typed scientific requests and deterministic runtime work without AI.
- Recheck [TypeSafe's official primitive contracts](https://docs.typesafe.ai/primitives), archive the relevant API schema/model identifier and validate service terms for intended payloads.
- Live credentials are optional for core implementation; live integration acceptance requires an authorized test account when available.

## Actionable backlog

- [ ] **S07-01 — Provider boundary** (AI): Implement `DecisionProvider` and a separate Rust HTTPS Jev adapter behind a feature flag. Pin API/request/response versions, returned concrete model ID and request rubric hashes. Keep service configuration outside scientific models.
- [ ] **S07-02 — Typed primitives** (AI): Implement `Choice<Route>` with exhaustive options/distribution, ordered `Score<Rubric>` with levels/legend, and `Noul` as a yes-probability. Validate finite ranges, option keys, probability sums and response/question alignment. Do not invent a confidence field for Noul.
- [ ] **S07-03 — Routing questions** (AI + Causal): Define narrow questions for association/intervention/counterfactual classification, uncertain-context detection, anomaly severity and escalation. Include unknown/ambiguous routes. Explicit typed requests always determine scientific operation.
- [ ] **S07-04 — Deterministic policy** (Platform): Implement a versioned Rust policy that thresholds advisory answers and checks allowable actions. Initial thresholds are marked provisional; calibration changes require review and a new policy revision. No model-proposed threshold or route can bypass schema/identification checks.
- [ ] **S07-05 — Fallback state machine** (Platform): On disabled service, absent credentials, timeout, rate limit or invalid response, dispatch valid typed requests normally; use deterministic schema/constraint triage; return `needs_explicit_query` for ambiguous free text. Preserve fallback reason without fake probabilities.
- [ ] **S07-06 — Network and data controls** (AI): Implement bounded deadlines/retries, cancellation, circuit behavior, payload minimization and credential redaction. `--offline` must prevent connection attempts. Remote payloads require explicit opt-in; never send raw restricted donor records by default.
- [ ] **S07-07 — Calibration corpus** (Evaluation): Build independently labeled routing/triage examples with held-out templates and ambiguous cases. Separate development, calibration and evaluation sets; include scientific query-level confusions and malformed inputs.
- [ ] **S07-08 — Calibration and baselines** (Evaluation): Report Choice accuracy/log loss, Noul Brier/log loss, Score ordinal error, reliability curves, abstention/coverage, escalation error and latency. Compare deterministic rules and simple classifiers. Provider confidence is not assumed calibrated for Cyra.
- [ ] **S07-09 — CLI and traceability** (Platform): Implement `route` and JSONL routing results with provider/model/policy revisions, scores, fallback status and actual selected deterministic action. Keep routing uncertainty outside scientific uncertainty fields.
- [ ] **S07-10 — Live adapter verification** (AI): When credentials are available, run a small authorized contract/correctness smoke test and record response/model metadata without secrets. If unavailable, mark live verification blocked; contract mocks alone do not complete this task.

## Deliverables

Typed primitive wrappers, optional Jev HTTPS adapter, deterministic policy/fallback module, calibration corpus/report and routing provenance schema. The [Choice](https://docs.typesafe.ai/primitives/choice), [Score](https://docs.typesafe.ai/primitives/score) and [Noul](https://docs.typesafe.ai/primitives/noul) contracts remain separately typed.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A07-01 | Valid Choice/Score/Noul fixtures and malformed distributions | Correct typed parsing; invalid ranges/options/sums rejected |
| A07-02 | Explicit counterfactual request receives an association suggestion | Original request semantics retained; advisory conflict recorded |
| A07-03 | Jev disabled, unreachable, rate-limited, slow or returns bad JSON | Offline simulation succeeds identically; fallback reason visible |
| A07-04 | Ambiguous free text with no usable provider answer | `needs_explicit_query`; no unintended experiment or invented confidence |
| A07-05 | Provider suggests changing thresholds/approving graph/creating measurement | Deterministic policy denies authority; no mutation |
| A07-06 | Run held-out routing/calibration suite against baselines | Actual error/calibration/coverage/latency reported; failures not hidden by aggregate accuracy |
| A07-07 | Run simulation with and without AI features, same manifest | Scientific output identity preserved; routing metadata may differ separately |
| A07-08 | Inspect `--offline` traffic and error logs | Zero external requests and no credentials or restricted raw records leaked |
| A07-09 | Authorized live request against pinned documented API | Valid actual response and model provenance; otherwise acceptance remains blocked |

## Evidence to collect

Store `evidence/sprint-07/<run-id>/` with contract snapshots, fixture hashes, policy revision, timeout/fallback traces, held-out calibration results, baseline comparisons and redacted live integration evidence. Current evidence: **none**. Synthetic/mock and live evidence must be labeled separately.

## Limitations and risks

Jev judges typed questions but is not a causal identification oracle. Provider/model changes can invalidate calibration. A score on a rubric is not a biological assay or uncertainty estimate. Service unavailability must never block scientific execution; it may block the live-adapter completion gate. This sprint cannot be signed fully complete on mock tests alone.

## Completion sign-off

- [ ] S07-01 through S07-10 completed; A07-01 through A07-09 passed.
- [ ] Deterministic authority and fallback audited; thresholds carry calibration status.
- [ ] Live verification evidence is present, or sprint remains explicitly blocked/incomplete.

Engineering approver: **pending**. Evaluation/scientific reviewer: **pending**. Software/provider/policy revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
