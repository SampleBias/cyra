# Cyra — Cybernetic Cellular Intelligence

Cyra is a headless Rust research platform being built to represent MSC state, simulate explicit mechanisms, and evaluate causal hypotheses. Its command-line interface is `vmcell`, retaining the interface name from the original VMSC directive.

**Sprint 00 is implemented: Rust workspace, CLI/JSONL foundation, scientific types, schemas and artifact storage. Sprint 01 measurement ingestion is implemented: CSV/TSV/JSON import, uncompressed Parquet and Zarr profiles, lineage, donor splits and an acquisition register with nothing approved.** Biological simulation is not implemented. Independent review remains pending. Sprints 02–11 are unstarted; no validated physiological prediction is claimed.

## Run the foundation

Reference target: Linux x86_64, Rust 1.97.1. The first build may acquire crates; the binary needs no network or interpreter.

```sh
cargo build --release --locked
./target/release/vmcell version
./target/release/vmcell --offline --format json capabilities
./target/release/vmcell --format json schema request
printf '%s\n' '{"schema_version":"vmcell.protocol.v1","id":"q-1","operation":"version","arguments":{}}' | ./target/release/vmcell agent --protocol jsonl
```

Available operations are `version`, `capabilities`, `schema`, `import`, measurement `observe`, `provenance`, and the `agent` transport. `observe` retrieves stored measurements; it does not estimate an association. Planned scientific commands return unsupported with exit 3 and no placeholder prediction.

In this managed workspace, use `CARGO_HOME="$PWD/.build/cargo" cargo build --release --locked --offline` for the prepared writable dependency cache. That local path is an environment convenience, not a project requirement.

See [contributor instructions](CONTRIBUTING.md), [Sprint 00 implementation evidence](docs/SPRINT_00_IMPLEMENTATION.md), [Sprint 01 implementation evidence](docs/SPRINT_01_IMPLEMENTATION.md) and the [project-name decision](docs/adr/0002-project-identity.md).

## Start here

1. Read the [engineering specification](docs/SPECIFICATION.md) for scope, architecture, and scientific contracts.
2. Use the [roadmap](docs/ROADMAP.md) to follow dependencies and milestones.
3. Execute [Sprint 00](docs/sprints/00-foundation.md), then work through the numbered sprint files.
4. Check the [acceptance traceability matrix](docs/TRACEABILITY.md) before closing a milestone.

Supporting contracts:

- [Scientific governance and completion rules](docs/GOVERNANCE.md)
- [CLI and JSONL interface](docs/CLI_CONTRACT.md)
- [Initial terminal demonstration and reproduction](docs/DEMO.md)
- [References and evidence register](docs/REFERENCES.md)

## Planning authority

This backlog translates the original VMSC directive supplied on 2026-10-22 and the subsequent rename to Cyra. No separate accompanying specification or sprint files were present. Reconcile any later supplied authoritative documents explicitly before changing scope. No public distribution license has been selected; workspace packages remain nonpublishable pending the owner's release-license decision.

The directive takes precedence over these documents. The specification defines shared contracts; sprint files define executable work and evidence for those contracts. Changes require an architecture decision record and updates to affected tests and sprint items. Writing a plan or passing a synthetic test does not constitute scientific validation.

## Delivery boundary

The deterministic scientific runtime must build and execute using Rust without Python or JavaScript. Optional AI adapters cannot block offline computation. Biological prediction claims remain unvalidated until independent experimental benchmarks support a narrowly stated claim.
