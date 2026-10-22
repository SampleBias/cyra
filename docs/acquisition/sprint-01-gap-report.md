# Sprint 01 acquisition gap report

Reviewed 2026-10-22 from public repository and paper pages. Nothing in this report was downloaded, hashed, or approved for a benchmark. Independent steward sign-off is pending. The machine-readable list is [sprint-01-register.json](sprint-01-register.json). `approved` is empty.

## What would be required

An independent benchmark dataset for the later evaluation sprints needs paired transcript, secreted-protein, and functional-assay measurements from Wharton's jelly MSCs, with donor identifiers, passage, batch, an open reuse license, and raw matrices this project can hash. No reviewed accession meets that bar.

## Candidates

These remain candidates. A page that states a donor count is not a Cyra census of the raw files.

| Accession | What the reviewed page supports | Why it is not approved |
| --- | --- | --- |
| GSE269586 | GEO series for WJ-MSC RNA-seq and MACE-seq during GMP manufacturing. The Cytotherapy article also discusses the secretome. | Donor count was not in the reviewed excerpts. The GEO accession is not treated as a proteomics or functional-assay matrix. Rights were not confirmed. Files were not hashed. |
| CNP0000562 | Sun et al. 2020, DOI `10.1186/s13287-020-01660-4`, deposit scRNA-seq of primary WJ-MSCs from three donors. Methods also describe bulk RNA-seq. | Donor identifiers and passage were not extracted. License and file hashes are absent. |
| GSE110791 | GEO single-cell comparison of hWJSCs and hBMMSCs. A citation in Sun et al. says 103 hWJSC from three cords and 63 hBMMSC from two donors. | Those counts were not recomputed here, so the register leaves `donor_count` null. Rights and hashes are absent. |
| PXD064143 | PRIDE secretome of WJ-MSCs primed with IL-1β, TNF-α, and IL-17 at 3, 6, and 12 ng for 4, 24, and 48 h. Passage 4 is in the sample protocol. Paper DOI `10.1016/j.bbrc.2025.152391`. | Donor count, license, and repository-support statement were not confirmed in the reviewed summary. Functional results in the paper are not a hashed assay matrix. |
| PXD063958 | ProteomeXchange/PRIDE intracellular proteome across isolation methods and FGF2. DOI `10.6019/PXD063958`. The repository summary says the dataset is supported by the repository. | Donor count and license were not confirmed. Files were not hashed. |
| MSV000096363 / PXD057695 | MassIVE DOI `10.25345/C5F47H59C` says CC0 1.0, Partial Public, 3 donors, passages P3/P5/P10, collection at 24/48/72 h, 3 technical runs, and 81 samples. | Files were not retrieved. Partial Public was not tested as anonymous access to every file. A different accession, PXD056000, uses a similar title and says 2 donors. This set is secretome-only. |

## Rejected as benchmark sources

- **GSE35585 / GSM2400244.** The series is ENCODE/SUNY Albany RIP-seq. The Wharton's jelly sample reviewed here is one newborn male umbilical cord, not a multi-donor cohort with the required paired assays.
- **GSE76081.** Sun et al. use it as adipose MSC bulk RNA-seq. It is not a Wharton's jelly dataset.

## Consequence

Biological benchmarks that need an independent licensed cohort stay blocked. Synthetic fixtures in the Sprint 01 tests check the pipeline only. They are excluded from benchmark plans unless a test deliberately sets the eligibility gate, and even that gate fixture is not listed here as approved data.
