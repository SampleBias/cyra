# Sprint 01 — Biological data ingestion and provenance

Status: **In progress — engineering implemented; independent sign-off pending**. Accountable implementation owner: **implementation agent**. Independent reviewers: data steward and evaluation lead, not yet assigned. Milestone: **M1**, first half. See the [implementation report](../SPRINT_01_IMPLEMENTATION.md).

## Objective

Ingest measurement data without losing modality, missingness, donor grouping, units or provenance. Establish the data eligibility process early enough to expose independent-benchmark blockers.

## Dependencies and entry criteria

- Sprint 00 schemas, immutable artifact primitives and CLI/JSONL dispatcher pass acceptance.
- Storage dependency spike confirms a Rust-only supported configuration.
- All included test datasets have explicit synthetic origin or documented reuse rights.

## Actionable backlog

- [x] **S01-01 — Measurement schemas** (Data): Implement `MeasurementRecord`, `SampleRecord`, `DatasetManifest`, `SourceRecord`, `LicenseRecord` and `TransformRecord`. Include donor, cell/population, replicate, passage, tissue, batches, assay conditions/time, units, accession/version and checksums.
- [x] **S01-02 — Modality and missingness** (Data + Biology): Add distinct transcript, intracellular protein, secreted protein, metabolite and functional-assay modalities. Encode missing reasons, below-detection censoring and QC failures. Keep count/expression units separate from protein concentration and activity.
- [x] **S01-03 — Explicit import mappings** (Data): Implement CSV/TSV/JSON ingestion with user-supplied column/analyte/units mappings. Validate required metadata and referential integrity before committing. Quarantine incomplete records with reasons; never infer unknown donor IDs or silently repair units.
- [x] **S01-04 — Parquet tables** (Data): Write/read versioned measurement tables in bounded row groups. Preserve nulls, numeric precision and nested metadata references. Pin the allowed feature/codec profile and document schema migrations.
- [x] **S01-05 — Zarr arrays** (Data): Store omics matrices and trajectories with explicit axes, chunk shapes, units and validity/censoring masks. Implement bounded slice reads and reject unsupported codecs/version variants. Do not interpret absent chunks as measured zeros.
- [x] **S01-06 — Transactional ingestion and lineage** (Data): Stage originals/normalized output, hash both, create an immutable input/result manifest pair and atomically publish. Deduplicate by content without merging distinct biological samples. Record transformations and importer revision.
- [x] **S01-07 — Licenses and accessions** (Steward): Implement `eligible`, `quarantined` and `ineligible` statuses with rights basis and intended use. Unknown license/accession requirements cannot be bypassed silently. Keep dataset, code and model-weight licenses distinct.
- [x] **S01-08 — Split scaffolding** (Evaluation): Create deterministic donor-grouped split manifests; place technical replicates and repeated donor samples together. Track batch, passage, perturbation and study domains for later evaluation. Report unknown donor/overlapping-source eligibility failures.
- [x] **S01-09 — CLI and provenance inspection** (Data): Implement `import`, measurement retrieval under `observe`, `provenance` and matching JSONL operations. Return source-to-result lineage and explicit unavailable fields without accessing external services.
- [x] **S01-10 — Benchmark acquisition register** (Steward + Biology): Search/curate candidate independent datasets during execution. Record real accessions, modalities, donor counts, perturbations, passage/batch coverage, rights, raw-data accessibility and suitability. Publish a gap report; do not invent datasets if suitable data are unavailable.

## Deliverables

Versioned measurement schemas, Rust ingestion library, Parquet/Zarr storage profiles, synthetic fixtures with deliberately incomplete records, immutable lineage manifests, donor split manifests and an acquisition/eligibility register.

## Acceptance tests

| ID | Test | Required result |
| --- | --- | --- |
| A01-01 | Round-trip each modality through both supported storage layouts | IDs, units, values, null reasons and censoring survive exactly or within declared representation tolerance |
| A01-02 | Read small slices from a many-chunk synthetic array | Only needed chunks are accessed; memory bounded by documented working set |
| A01-03 | Import absent values, zero counts and below-detection values | Three distinguishable results; no implicit imputation or zero filling |
| A01-04 | Import incompatible units, duplicate biological IDs and orphan metadata | Clear validation failures with no successful dataset manifest |
| A01-05 | Interrupt ingestion or corrupt a source/normalized chunk | Atomic publication and checksum checks prevent an apparently valid dataset |
| A01-06 | Dataset has unknown/restricted rights or unresolved accession requirements | Explicit eligibility failure/quarantine; excluded from benchmark plans |
| A01-07 | Split donors with many cells, repeated samples and technical replicates | No donor crosses donor-holdout boundaries; stable assignment with recorded seed |
| A01-08 | Read a prior schema version and attempt an unsupported future version | Explicit migration with lineage for the former; actionable rejection for the latter |
| A01-09 | Inspect provenance via human CLI and JSONL | Same underlying lineage, license and origin information; machine output remains clean |

## Evidence to collect

Store `evidence/sprint-01/<run-id>/` with source and output hashes, round-trip reports, chunk-access/memory measurements, interrupted-import results, split leakage report and license eligibility decisions. The acquisition register must label candidates versus approved data. Engineering evidence is in [the Sprint 01 run](../../evidence/sprint-01/2026-10-22-data-provenance/manifest.json). It does not approve a dataset or sign the sprint off.

## Limitations and risks

Initial import coverage is explicit tabular/array schemas, not universal support for every omics format. Population-to-cell attribution and missing donor metadata cannot be repaired by a model. Accessible papers may lack reusable raw data. Block affected biological benchmarks if independent, licensed data cannot be obtained; synthetic fixtures only validate the pipeline.

## Completion sign-off

- [ ] S01-01 through S01-10 completed; A01-01 through A01-09 passed.
- [ ] Data steward approved schemas, provenance and rights policy.
- [ ] Acquisition gaps are recorded with owners, even if experimental data are not yet available.

Engineering approver: **pending**. Data/scientific reviewer: **pending**. Software/schema revision: **pending**. Evidence manifest: **pending**. Date: **pending**. Exit decision: **not signed off**.
