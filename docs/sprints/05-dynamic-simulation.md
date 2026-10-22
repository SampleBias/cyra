# Sprint 05 — Time-dependent mechanistic simulation

Status: **Not started**. Accountable owner: **Numerical methods lead** (named assignee pending). Reviewers: causal engineer and computational biologist. Milestone: **M3**, first half.

## Objective

Evolve typed cell/population states over time with controlled numerical error, explicit interventions and reproducible manifests. Numerical failure must be visible rather than disguised as a biological prediction.

## Dependencies and entry criteria

- Sprints 02 and 04 accepted; state units, structural mechanisms and result metadata stable.
- Solver/dependency selection respects the Rust-only default runtime.
- Reference target, operational budgets and reproducibility tier recorded in Sprint 00.

## Actionable backlog

- [ ] **S05-01 — Dynamics API** (Numerical): Implement state vectors with named units, `dx/dt=f(x,c(t),theta)`, parameter bundles, derivative evaluation and observation operators. Expose transition dependencies consistently with time-unrolled causal graphs.
- [ ] **S05-02 — Reference integrator** (Numerical): Implement or adopt a Rust fixed-step RK4 path for transparent verification; record step size, ordering and floating-point policy. Avoid claiming it is suitable for all stiff biochemical systems.
- [ ] **S05-03 — Adaptive integration** (Numerical): Select/implement a Rust adaptive solver, error estimator, state-scaled absolute/relative tolerances, minimum step, maximum work and event localization. Return diagnostic status for divergence, stiffness-like failure or non-finite state.
- [ ] **S05-04 — Event scheduler** (Numerical + Causal): Implement timed input pulses, state clamps, mechanism replacements and culture events. Specify event ordering at equal timestamps, interval boundary conventions and restoration after a clamp. Exactly split steps at discontinuities.
- [ ] **S05-05 — Constraint enforcement** (Biology + Numerical): Define nonnegative concentrations/counts and bounded fractions. Use documented invariant-preserving formulations or reject/reduce steps. Any tolerance-level correction is reported; substantial invalid states cause failure rather than silent clipping.
- [ ] **S05-06 — Stochastic ensembles** (Numerical): Add seeded draws from declared initial/exogenous/parameter distributions, stable per-cell/per-run substreams and explicit aggregation. Distinguish epistemic parameter ensembles from cell heterogeneity. Do not call this SDE support.
- [ ] **S05-07 — State/population coupling** (Biology): Define how cell states generate aggregate fluxes and observations. Include counts and culture volume explicitly, prevent double scaling and preserve single-cell versus population outputs.
- [ ] **S05-08 — Checkpoint and artifacts** (Platform): Write chunked trajectories, event logs, solver diagnostics and immutable run manifests. Support checkpoint/restart on the declared deterministic reference configuration, including RNG state and pending events.
- [ ] **S05-09 — Simulation and reproduction commands** (Platform): Implement `simulate`, dynamic intervention schedules, `reproduce` and JSONL equivalents. Verify all referenced hashes before replay; write new execution records without overwriting originals.
- [ ] **S05-10 — Numerical verification and budgets** (Numerical): Benchmark analytic decay, activation and population growth fixtures. Measure convergence, latency/memory and ensemble reproducibility using frozen workload sizes; record limitations and supported tolerances.

## Deliverables

Rust numerical engine, solver/configuration schema, event schedule format, trajectory storage, checkpoint/replay support, ensemble sampling and analytic verification report.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A05-01 | Exponential decay and constant production/clearance | Agreement with analytic solutions within stated state-specific tolerances |
| A05-02 | Halve RK4 step size on a smooth analytic fixture | Observed convergence consistent with method order before roundoff dominates |
| A05-03 | Tighten adaptive tolerances on activation/growth fixtures | Errors meet documented criteria and diagnostics report actual work |
| A05-04 | Pulse/clamp at non-grid time and two simultaneous events | Correct boundary/state continuity or jump; deterministic documented event order |
| A05-05 | Negative concentration, fraction >1, NaN and solver budget exhaustion | Explicit failure/rejected step; no silently clamped successful physiological output |
| A05-06 | Same seed/configuration with stable ensemble substreams | Same scientific artifacts on reference target; changed seed affects stochastic draws only |
| A05-07 | Doubling cell number or volume in a simple secretion fixture | Correct dimensional scaling with no duplicate population factor |
| A05-08 | Interrupted run resumed from checkpoint | Agrees with uninterrupted reference execution; incomplete run never presented as complete |
| A05-09 | Replay manifest and then tamper with an input | Original reproduces; tampered input fails integrity checks before simulation |
| A05-10 | Request unsupported dynamic counterfactual abduction | Returns unsupported while ordinary simulation remains usable |

## Evidence to collect

Store `evidence/sprint-05/<run-id>/` with analytic curves/error summaries, convergence results, event traces, solver diagnostics, reference hardware, latency/memory measurements and replay/checkpoint reports. Record numerical and stochastic uncertainty separately. Current evidence: **none**.

## Limitations and risks

RK4 and an explicit adaptive method do not cover all stiff systems. Unsupported regimes must fail visibly until a suitable solver is verified. Ensemble variation is conditional on chosen distributions and does not establish calibrated biological uncertainty. Dynamic counterfactuals remain unavailable unless a supported observation/abduction model is explicitly attached.

## Completion sign-off

- [ ] S05-01 through S05-10 completed; A05-01 through A05-10 passed.
- [ ] Units, invariants, event semantics and numerical tolerance policy reviewed.
- [ ] Reference replay succeeds without network or AI credentials.

Engineering approver: **pending**. Numerical/scientific reviewer: **pending**. Software/solver revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
