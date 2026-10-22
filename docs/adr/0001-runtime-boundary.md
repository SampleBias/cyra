# ADR 0001 — Rust runtime and foundation dependency boundary

Date: 2026-10-22. Status: implemented engineering decision; independent review pending. Owner: Codex implementation agent under the user's authorization to begin Sprint 00.

## Decision

Use Rust 2024, compiler/MSRV 1.97.1, with `cyra-core`, `vmcell` and development-only `xtask`. Pin direct versions and Cargo.lock. Reference target: Linux x86_64 CPU. No scientific libraries, network clients, AI SDKs or interpreters are needed by this foundation.

| Dependency | Version/features | License | Boundary |
| --- | --- | --- | --- |
| serde | 1.0.229; derive | MIT OR Apache-2.0 | Typed serialization; Rust derive steps |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | JSON with Cyra duplicate/depth checks |
| schemars | 1.2.2 | MIT | Schemas generated from Rust types |
| clap | 4.6.7; std/derive/help/usage/error-context; defaults off | MIT OR Apache-2.0 | CLI; color/suggestions omitted |
| sha2 | 0.10.9; force-soft | MIT OR Apache-2.0 | Software SHA-256; assembly excluded |
| rustix | 1.1.4; fs | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Safe OS operations; no scientific FFI |
| tempfile | 3.27.0; test-only | MIT OR Apache-2.0 | Temporary tests; application creates private stores |

Exact transitive versions/licenses and Rust build scripts appear in the evidence inventory. Normal system C runtime linkage is permitted; scientific computation is not delegated to C/C++ libraries.

## Future dependency spikes

Units: evaluate `uom` or a documented internal dimensional system in Sprint 02. No units library selected or benchmarked.

RNG: locally inspected `rand_chacha` 0.3.1 (MIT OR Apache-2.0) is a candidate. Record algorithm/stream/state in Sprint 05. No scientific RNG is needed for current dispatch/hashing; test temporary-file randomness is not a scientific seed.

Linear algebra: locally inspected `nalgebra` 0.33.3 (Apache-2.0) is a candidate without native acceleration. Validate decomposition/rank behavior in Sprint 04; no solver has been approved now.

ODE: a Rust RK4 reference and separately verified adaptive method remain Sprint 05 work. Selection includes convergence, stiff-regime failures and event handling.

Storage: the Sprint 01 decision is [ADR 0003](0003-storage-profiles.md). It pins uncompressed `parquet` 60.0.0 with only the `arrow` feature and an in-tree Zarr v2 subset. The `zarrs` crate and compressed Parquet codecs were not adopted.

## Consequences

Scientific core has no AI dependency. Later commands return discoverable unsupported statuses. Descriptor-relative file operations narrow current platform support until equivalent implementations are tested. `xtask audit` is a dependency policy check, not a security-advisory scan.

Original source is MIT, as recorded in `LICENSE`. Third-party dependencies keep their own licenses. Packages use `publish=false` and are not published to crates.io.
