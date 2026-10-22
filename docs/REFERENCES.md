# References and evidence register

Reviewed for planning on 2026-10-22. These references inform architecture and future curation; they are not fitted parameter datasets or completed benchmarks. Exact library versions, feature flags and API snapshots must be pinned and rechecked in their implementation sprints. No full-text biological parameter extraction was performed for this plan.

## Causal theory

**CAUSAL-01 — Judea Pearl, Causality.** The [author-hosted table of contents](https://bayes.cs.ucla.edu/BOOK-99/book-toc.html) is the user-specified reference. Chapter 1 covers graph and structural-model foundations; Chapter 3 concerns intervention and identification; Chapter 7 concerns counterfactual semantics. Implementers must cite the edition and exact definition/theorem consulted in algorithm design records; the linked contents page is an index, not a proof specification. Acceptance examples in this backlog are constructed synthetic fixtures, not findings from the book or from cells.

## Biological curation starting points

**BIO-01 — Dominici et al. (2006).** [Minimal criteria for defining multipotent mesenchymal stromal cells](https://pubmed.ncbi.nlm.nih.gov/16923606/), DOI `10.1080/14653240600855905`. This position statement describes culture adherence, a surface-marker panel and differentiation assays for MSC characterization. Use it as a historical identity reference, with assay context; it does not establish a quantitative WJ-MSC simulator. Abstract metadata reviewed.

**BIO-02 — Valencic et al. (2010).** [The immunosuppressive effect of Wharton's jelly stromal cells depends on the timing of their licensing and on lymphocyte activation](https://pubmed.ncbi.nlm.nih.gov/20078387/), DOI `10.3109/14653240903493417`. The study motivates recording licensing timing, target-cell activation and contact conditions. It does not supply a universal conversion from secreted concentration to immunomodulatory potency. Abstract reviewed; full methods, data rights and numeric parameter extraction remain Sprint 06 work.

**BIO-03 — WJ-MSC functional study (2015).** [Immunosuppressive properties of Wharton's jelly-derived mesenchymal stromal cells in vitro](https://pubmed.ncbi.nlm.nih.gov/26228529/). This study motivates distinguishing expression, intracellular enzyme-related mechanisms, soluble factors and contact-dependent functional assays. Review the complete methods and underlying data before selecting it for calibration or evaluation. Abstract reviewed; no numeric parameter adopted.

**BIO-04 — Definition review candidate (2024).** [Delphi-driven consensus definition for mesenchymal stromal cells and clinical reporting guidelines for mesenchymal stromal cell-based therapeutics](https://pubmed.ncbi.nlm.nih.gov/39580717/). Search metadata located; full content not reviewed. Sprint 02 must review applicability and reconcile newer guidance with BIO-01 rather than treating a historical checklist as exhaustive.

## Jev primary documentation

**JEV-01 — TypeSafe primitives.** [Official primitive overview](https://docs.typesafe.ai/primitives). Use the documented request/answer contract and revalidate it during Sprint 07. Provider descriptions of reliability are not Cyra calibration evidence.

- [Choice](https://docs.typesafe.ai/primitives/choice): categorical selection with a distribution and confidence.
- [Score](https://docs.typesafe.ai/primitives/score): ordered rubric position with a distribution, legend and confidence; it is not a biological measurement scale.
- [Noul](https://docs.typesafe.ai/primitives/noul): probability of a yes/no proposition; there is no separate confidence field.

Pin the provider's concrete model identifier returned by the service, contract snapshot and request rubric. Avoid floating aliases in reproducible integration evidence. A Rust HTTPS adapter avoids any requirement for a non-Rust provider SDK.

## Rust dependency candidates

**RUST-01 — Zarr.** [zarrs crate documentation](https://docs.rs/zarrs/latest/zarrs/) describes Rust multidimensional-array storage. Sprint 01 did not adopt `zarrs`. The implemented profile is the in-tree uncompressed Zarr v2 subset `cyra.zarr.v1` in [ADR 0003](adr/0003-storage-profiles.md). A missing chunk is an error, not a measured zero.

**RUST-02 — Parquet.** [Apache Parquet Rust crate documentation](https://docs.rs/parquet/latest/parquet/) is the table backend. Sprint 01 pins `parquet` 60.0.0 with default features off and only `arrow`, uncompressed, PLAIN encoding, and 256-row groups. See [ADR 0003](adr/0003-storage-profiles.md).

**RUST-03 — Local inference.** [Candle core documentation](https://docs.rs/candle-core/latest/candle_core/) is a starting point for the optional Rust inference spike. A framework's existence does not establish compatibility with a particular reasoning model, tokenizer, quantization or license. Sprint 08 must demonstrate a pinned CPU path and document optional acceleration separately.

## Curation record required during execution

For every adopted source, record a stable ID; primary URL/DOI/accession; retrieved date; version; rights; organism/tissue; donor and passage scope; perturbation/assay context; exact supported claim; page/table/figure locator; extraction method; reviewer; and conflicting evidence. Preserve immutable source hashes where permitted. Clearly mark inaccessible sources, abstract-only reviews and secondary summaries. Do not invent source accession numbers for future benchmark datasets.
