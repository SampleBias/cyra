# Sprint 01 implementation and verification

Project: **Cyra — Cybernetic Cellular Intelligence**. Date: 2026-10-22. Implementation owner: the implementation agent, continuing from the Sprint 00 foundation. Independent engineering and scientific review: pending. This report concerns measurement ingestion and provenance. It is not a physiological model and it approves no external dataset.

## Implemented scope

- `cyra-data` ingests explicit CSV, TSV and JSON tables into a private dataset directory.
- Measurement records keep transcript, intracellular protein, secreted protein, metabolite and functional-assay modalities distinct. Measured zero, `not_measured`, below-detection censoring and QC failure stay distinct. Units convert only through an explicit positive factor of the same quantity kind.
- Unknown donors are quarantined and are not inferred. Conflicting duplicate biological keys abort with no manifest. Identical content is stored once. Distinct sample identifiers are not merged.
- `cyra.parquet.v1` stores canonical decimal strings in uncompressed row groups of at most 256 rows. `cyra.zarr.v1` is an in-tree Zarr v2 subset with 32-observation chunks. A missing or short chunk is an error, not a zero. See [ADR 0003](adr/0003-storage-profiles.md).
- Publication is staged and renamed into place. The result manifest hashes every other file and does not hash itself. Stopping before publication removes the staging directory. An existing output is not overwritten. A declared source checksum must match the input bytes; the file is not altered to force a match.
- Dataset, code and model-weight licenses are separate records. Unknown rights are quarantined. Restricted rights or an unresolved required accession are ineligible. Either outcome can publish a manifest with exit 5 and is excluded from benchmark plans. Synthetic fixtures are not benchmark datasets.
- Donor splits are a SHA-256 bucket of `cyra.donor-split.v1`, the seed and the donor id. Cells, technical replicates and repeated samples of one donor share a split. Unknown donors and overlapping source identities make the split unusable for donor holdout.
- `vmcell import`, `observe` and `provenance` use the same functions over the CLI and JSONL. Measurement retrieval is not an association estimate. Empty `observe` arguments remain unsupported. `cyra.measurement.v0` migrates explicitly; `cyra.measurement.v2` is rejected.
- The [acquisition register](acquisition/sprint-01-register.json) lists reviewed public accessions as candidates or rejected sources. `approved` is empty. The [gap report](acquisition/sprint-01-gap-report.md) states that no paired transcript, secreted-protein and functional-assay cohort with donor identifiers, passage, batch, open rights and hashed raw files was obtained.

No simulator, causal engine, profile bundle, fit or biological benchmark was added.

## Evidence location

The engineering run is [evidence/sprint-01/2026-10-22-data-provenance/manifest.json](../evidence/sprint-01/2026-10-22-data-provenance/manifest.json). It records command outputs, the lockfile and binary hashes, the dependency audit, round-trip, chunk-access, interrupted-import, split and license reports, and the acquisition register hash. The audit covers the host resolve only. A Haiku-only `cc` build script remains in the lockfile through time-zone support and is not compiled for this host. `physiological_validation` is `not_performed`. Independent review is `pending`.

## Acceptance mapping

| Acceptance | What the tests check |
| --- | --- |
| A01-01 | Every modality round-trips through Parquet and Zarr with the same records, units and decimal spelling |
| A01-02 | A two-row slice of a 200-row array opens only chunk 0 and stays within the 3296-byte one-chunk working set; a missing chunk is not a zero |
| A01-03 | A stored zero, a not-measured absence and a below-detection censor limit remain three results |
| A01-04 | Incompatible units, conflicting duplicate keys, orphan samples, unknown analytes and modality conflicts publish no manifest |
| A01-05 | `publish: false` leaves no output or staging directory; a changed file fails its checksum; a mismatched declared checksum is not repaired |
| A01-06 | Unknown, restricted and unresolved-accession datasets are excluded from benchmark plans. The register's approved list is empty |
| A01-07 | The same seed repeats. One donor's cells, technical replicates and repeated samples share a split. Overlapping source identities are not assigned |
| A01-08 | v0 maps through a named migration. v2 is rejected and no rows are written |
| A01-09 | Human CLI and JSONL provenance report the same manifest hash, origin and license records. Empty `observe` still exits 3 |

Fixtures are synthetic. A test that sets `origin` to `experimental` with public-domain rights only exercises the eligibility gate. That table is not an approved cohort and is not in the acquisition register.

## Limitations and handover

Import covers explicit tabular mappings, not every omics container. Population measurements are not copied onto cells. Missing donors are not repaired. The acquisition gap blocks an independent biological benchmark that needs paired transcript, secreted protein and functional assays. Repository pages were reviewed on 2026-10-22; raw matrices were not downloaded, so donor counts that a page did not state stay null, and counts that a page did state were not re-derived from the files.

Sign-off boxes in the sprint file stay unchecked. No reviewer identity was invented. Source distribution licensing remains undecided. Sprint 02, ontology and profile bundles, is the next engineering package and is not started.
