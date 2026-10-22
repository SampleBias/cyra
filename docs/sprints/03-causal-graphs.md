# Sprint 03 — Typed causal graphs and identification

Status: **Not started**. Accountable owner: **Causal inference lead** (named assignee pending). Reviewers: causal methods reviewer and platform engineer. Milestone: **M2**, first half.

## Objective

Represent and validate causal assumptions explicitly, then answer supported graphical queries with inspectable certificates. Graphical validity is not evidence that the proposed biology is true.

## Dependencies and entry criteria

- Sprints 00 and 02 accepted; typed domains, units and provenance stable.
- Mathematical reference: [Pearl's author-hosted contents](https://bayes.cs.ucla.edu/BOOK-99/book-toc.html), especially Chapters 1 and 3. Record the actual edition and definitions consulted during implementation.
- Initial scope fixed to DAGs with explicit latent variables and finite time-unrolled graphs; advanced mixed-graph methods are not implicitly supported.

## Actionable backlog

- [ ] **S03-01 — Typed graph schema** (Causal): Implement node IDs, endogenous/exogenous/measurement roles, observed/latent status, domains/units, time-slice metadata, directed edges and edge assumption/source records. Distinguish sample-level, cell-level and population-level nodes.
- [ ] **S03-02 — Structural validation** (Causal): Detect missing/duplicate nodes, self-edges, directed cycles, malformed time transitions, incompatible domains and invalid measurement links. Require every equation's declared parents to match its graph parents.
- [ ] **S03-03 — Graph algorithms** (Causal): Implement deterministic topological ordering, ancestor/descendant sets and d-separation, using a validated traversal algorithm. Condition only on eligible observed variables in identification queries; latent nodes remain in path reasoning.
- [ ] **S03-04 — Modeled confounding report** (Causal): Report common ancestors and open back-door paths relative to treatment/outcome. Explain why a candidate variable is or is not an admissible adjustment variable. State that this identifies modeled confounding, not unknown biological confounders from data.
- [ ] **S03-05 — Back-door criterion** (Causal): Validate a supplied observed adjustment set, exclude treatment descendants under this criterion, and check blocked back-door paths. Enumerate candidate/minimal sets with explicit resource limits; output the adjustment estimand and assumptions.
- [ ] **S03-06 — Front-door criterion** (Causal): Implement the standard single-treatment, single-mediator, single-outcome graphical checks. Require interception of all directed treatment-outcome paths, no open treatment-mediator back-door path, and outcome-mediator back-door blocking by treatment. Output the supported estimand and positivity conditions.
- [ ] **S03-07 — Identification statuses** (Causal): Separate `identified`, `not_identifiable`, `unsupported` and algorithm resource limits. Implement at least one recognized non-identifiable fixture with a witness; a failed back-door search alone is not a non-identifiability proof.
- [ ] **S03-08 — Revision and approval skeleton** (Platform): Hash graph bundles; enforce draft/reviewed/approved/retired transitions and create-new-revision updates. Attach author, sources, scope, assumptions and review records. Approved files cannot be edited in place through the API.
- [ ] **S03-09 — CLI and certificates** (Platform): Implement `graph inspect`, `graph validate`, `graph identify` and JSONL services. Output human-readable paths and structured certificates that specify graph revision, query and algorithm version.
- [ ] **S03-10 — Biological draft graph** (Biology + Causal): Add the illustrative licensing/secretome transition graph and distinguish assumed from supported edges. Express feedback through time; do not label the draft biologically validated when its graph tests pass.

## Deliverables

Graph schema, Rust algorithms, identification report schema, immutable graph bundles, synthetic graph fixtures, approval skeleton and documented algorithm support matrix.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A03-01 | Chains `X→M→Y` and forks `X←C→Y` | Conditioning on the middle node blocks the path |
| A03-02 | Collider `X→K←Y` and descendant `K→D` | Unconditioned path blocked; conditioning on K or D opens it |
| A03-03 | Compare d-separation against exhaustive active-path checks on small DAGs | Agreement across generated acyclic cases and conditioning sets |
| A03-04 | `C→X`, `C→Y`, `X→Y` | `{C}` passes back-door; empty set fails; descendant/collider candidates are handled correctly |
| A03-05 | `U→X`, `U→Y`, `X→M→Y` with U latent | Standard front-door certificate succeeds; adding an unmediated `X→Y` edge invalidates it |
| A03-06 | Explicit latent confounding without an identifying design | Recognized fixture reports non-identifiability with witness; no numeric effect |
| A03-07 | Unsupported graph family or exhausted enumeration budget | `unsupported`/resource error, never an invented negative identification proof |
| A03-08 | Cycle, mismatched equation parents, invalid time edge and incompatible units | Reject with node/edge-specific diagnostics |
| A03-09 | Revise an approved model and inspect the old revision | New hash/review required; old model and its evidence remain unchanged |

For the standard discrete front-door case, the certificate targets `P(y|do(x)) = Σ_m P(m|x) Σ_x' P(y|m,x')P(x')` under its stated graphical and support assumptions. Numerical estimation is Sprint 04 work, not implied by a graph certificate.

## Evidence to collect

Store `evidence/sprint-03/<run-id>/` with graph fixtures, expected path/adjustment results, exhaustive cross-check reports, identification certificates, theorem/algorithm design notes and review records. Current evidence: **none**.

## Limitations and risks

No general hidden-confounder discovery, complete identification calculus or cyclic equilibrium support is promised. A structurally valid graph may be biologically wrong. Finite data can violate positivity even when the graph identifies a functional. Cross-scale edges require explicit aggregation/observation mechanisms.

## Completion sign-off

- [ ] S03-01 through S03-10 completed; A03-01 through A03-09 passed.
- [ ] Algorithm scope and non-identifiability claims reviewed by a causal methods reviewer.
- [ ] Biological graph remains correctly labeled as an assumption-bearing model.

Engineering approver: **pending**. Causal/scientific reviewer: **pending**. Software/graph revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
