# Developing Cyra

Use Rust 1.97.1, pinned by `rust-toolchain.toml`; this is also the current minimum supported version until a lower version is explicitly tested. Reference target: `x86_64-unknown-linux-gnu`. File-security primitives use Unix descriptor-relative operations; Windows support is not claimed.

## Build and check

```sh
cargo build --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo run --locked -p xtask -- audit
```

After dependencies are cached, add `--offline` to Cargo invocations. In this managed workspace set `CARGO_HOME="$PWD/.build/cargo"` per command to use the prepared writable cache. Do not change system Rust/Cargo configuration. The scientific runtime never invokes Cargo or an interpreter.

Tests cover malformed/duplicate/deep JSON, clean output, causal refusal metadata, finite values, distinct mathematical request types, deterministic canonical bytes, atomic publication, concurrent writes, symlink/traversal rejection and permissions. They are engineering checks, not physiological validation.

## Schemas and compatibility

```sh
cargo run --locked -p xtask -- schemas
```

Review generated schema changes, not just Rust diffs. Tests reject schema drift. `vmcell schema <name>` emits a schema in the standard envelope in JSON mode. Operation argument checks supplement the transport schema. Scientific-query types are reserved and have no execution semantics until their sprint is implemented.

## Evidence

```sh
cargo run --locked -p xtask -- evidence evidence/sprint-00/your-new-run-id
cargo run --locked -p xtask -- evidence-sprint-01 evidence/sprint-01/your-new-run-id
```

The Rust runner creates a new private directory, executes formatting/lint/tests/release build and dependency policy checks, records output hashes/timings, inventories sources, and tests the release binary with a cleared environment and unusable executable search path. It refuses an existing directory. Avoid source edits during collection. Never revise an old run to conceal a failure.

For an OS-isolated offline run after populating the cache:

```sh
unshare --user --map-root-user --net cargo run --offline --locked -p xtask -- evidence evidence/sprint-00/offline-new-run
```

This requires permitted Linux namespaces; unsupported hosts must report that limitation. The collector alone does not assert network isolation. Keep the outer command and namespace proof alongside its manifest. Reproduction by another reviewer is a separate event.

## Artifact boundary

`ArtifactStore` creates private directories, opens path components without following symlinks, writes staging files as `0600`, synchronizes content and publishes by atomic no-overwrite hard link. Canceled staging removes the temporary file. An abrupt kill may leave unpublished `.stage-*` files, which never count as committed results. Identical publication returns `AlreadyExists`.

Verified reads check SHA-256 and length. Filesystem owners can alter files outside the API; corruption is detected, not prevented by a claim of filesystem immutability. Do not import arbitrary archive hard links or give model-generated paths ambient filesystem access.

Canonical `cyra-json-v1` sorts keys recursively, retains array order, emits compact UTF-8 without a newline and uses pinned serde_json number spelling. It is not RFC 8785. Execution timestamps/duration are excluded from future scientific result identity.

## Dependency and review policy

`xtask audit` checks locked dependency licenses, rejects declared native links and known native/interpreter build packages, and reports Rust build scripts for review. It is not a vulnerability database audit. Dependency/feature changes require feature-tree, license/native-build and regression review. No unreviewed network, Python, JavaScript, BLAS or AI runtime belongs in the default scientific dependency path.

Keep approved models immutable; proposals and measurements remain separate. Source or label every biological parameter. Record decisions as ADRs and never convert engineering test success into a biological claim. No distribution license has been granted; packages and restricted data must not be published.
