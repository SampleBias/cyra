# ADR 0003 — Uncompressed measurement storage profiles

Date: 2026-10-22. Status: implemented engineering decision; independent review pending. Owner: implementation agent under the user's authorization to build Sprint 01. Reviewers: pending.

## Problem and evidence

Sprint 01 must store measurement tables and array slices without a native compression library, without treating a missing chunk as a measured zero, and without turning decimal text into binary floating point. ADR 0001 left the storage spike open. The `zarrs` crate can pull codec implementations that are outside the Rust-only default. Parquet's default feature set enables several compressors.

## Decision and alternatives

Use two explicit profiles. Both store numbers as canonical base-10 strings. Trailing fractional zeros are insignificant. JSON numbers are rejected at import. There is no `f64` measurement column.

`cyra.parquet.v1` uses `parquet` 60.0.0 with default features disabled and only the `arrow` feature, plus `arrow-array` and `arrow-schema` 60.0.0. Writers set uncompressed codec, dictionary encoding off, PLAIN encoding, and at most 256 rows per row group. File metadata records the profile and codec. Readers reject any other profile, a v0 file, a future schema, and any compressed column.

`cyra.zarr.v1` is an in-tree Zarr v2 directory subset, not the `zarrs` crate. Groups declare `zarr_format` 2, compressor null, filters null, and C order. Each observation chunk holds 32 rows. Value and censor planes are fixed 48-byte UTF-8 rows. Presence, missing-reason, and censor-flag arrays are separate. A missing or short chunk is an error. Fill values are not returned as measurements. A chunk file is counted once per slice even when several rows in that chunk are requested. The documented one-chunk working set is `32 * 48 * 2 + 32 * 2 * 2 + 32 * 3` bytes (3296), covering the value plane, censor plane, two length arrays, and three mask arrays.

Dataset directories are not the flat artifact store from Sprint 00. A private staging directory is renamed into place only after the input manifest, originals, Parquet file, Zarr arrays, quarantine list, optional splits, and result manifest exist. The result manifest hashes every other file and does not hash itself. `publish: false` deletes the staging directory and does not create the output. An existing output path is not overwritten. Declared source checksums are compared with the input bytes before any dataset write; the file is not altered to match a declared hash.

Rejected alternatives: installing `zarrs` or enabling Parquet compression features; storing IEEE-754 values; interpreting Zarr fill or a missing chunk as zero; merging distinct sample identifiers that happen to carry the same concentration.

## Scientific and compatibility effects

These profiles preserve modality, units, measured zero, `not_measured`, and below-detection censor limits as different stored states. They do not impute, identify an effect, or validate physiology. A published manifest can still be quarantined or ineligible, and a synthetic fixture is not a benchmark dataset. Supported measurement documents are `cyra.measurement.v1` and an explicit migration from `cyra.measurement.v0`. `cyra.measurement.v2` is rejected and is not partially read.

Schemas: `import-mapping`, `dataset-manifest`, `observation-request`, and `measurement-record`. Tests: A01-01 through A01-08. Native packages `cc`, `cmake`, and compression sys-crates remain banned when they appear in the host dependency audit.

## Review

Evidence manifest: [Sprint 01 engineering run](../../evidence/sprint-01/2026-10-22-data-provenance/manifest.json). The engineering checks recorded there passed. `xtask audit` reviews the host resolve only. A Haiku-only `cc` build script stays in the lockfile through time-zone support and is not compiled for this host. Reviewer/date: pending. Decision: not independently approved.
