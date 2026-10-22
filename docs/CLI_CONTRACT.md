# `vmcell` CLI and JSONL contract

Status: foundation v1 is implemented for `version`, `capabilities`, `schema` and `agent`. Sprint 01 adds `import`, measurement retrieval through `observe`, and `provenance`. Association estimands, profile inspection, graphs, simulation and later commands remain unsupported and return no computed number. Generated contracts are in `schemas/`; extend them compatibly. The project is Cyra — Cybernetic Cellular Intelligence; its CLI remains `vmcell`.

## Command surface

| Command | Purpose | Delivery sprint |
| --- | --- | --- |
| `vmcell version` | Software, schema, toolchain and feature information | 00 |
| `vmcell capabilities` | Supported operations/model families and unavailable adapters | 00, extended throughout |
| `vmcell schema <name>` | Emit a versioned JSON schema | 00, extended throughout |
| `vmcell init --profile <id> --out <dir>` | Materialize an immutable profile bundle | 02 |
| `vmcell import --input <path> --mapping <file> --metadata <file> --out <dir>` | Validate and ingest measurements | 01 |
| `vmcell graph inspect --model <file>` | List nodes, edges, equations and evidence | 03 |
| `vmcell graph validate --model <file>` | Validate structure, types, units and assumptions | 03 |
| `vmcell graph identify --model <file> --request <file>` | Return identification status and certificate | 03 |
| `vmcell observe --profile <file>` | Inspect modeled state, with origin labels | 02 |
| `vmcell observe --dataset <manifest> --request <file>` | Retrieve/describe measurements or a declared association | 01, 04 |
| `vmcell simulate --profile <file> --scenario <file> --seed <u64> --out <dir>` | Evolve a specified dynamic model | 05 |
| `vmcell intervene --model <file> --request <file> --seed <u64> --out <dir>` | Evaluate a supported structural intervention or contrast | 04; dynamic schedules 05 |
| `vmcell counterfactual --model <file> --request <file> --seed <u64> --out <dir>` | Perform supported abduction-action-prediction | 04 |
| `vmcell explain --result <file>` | Explain recorded computation deterministically | 04, 06 |
| `vmcell provenance --artifact <path>` | Resolve lineage, evidence and licensing | 01 |
| `vmcell reproduce --manifest <file> --out <dir>` | Verify inputs and rerun into a new result directory | 05; complete demo 06 |
| `vmcell route --request <file>` | Return advisory routing and fallback provenance | 07 |
| `vmcell reason --task <file> --out <dir>` | Produce a structured advisory result/proposal | 08 |
| `vmcell model propose/review/approve ...` | Explicit review workflow, role-scoped; never AI-callable approval | 03 skeleton, 08 full |
| `vmcell fit --plan <file> --out <dir>` | Fit under frozen training and validation rules | 09 |
| `vmcell evaluate --plan <file> --out <dir>` | Evaluate frozen artifacts and baselines | 09 synthetic, 10 biological |
| `vmcell agent --protocol jsonl` | Serve all permitted operations over stdin/stdout | 00 skeleton, complete 11 |

Global options include `--format human|json` and `--offline`. Human output may use headings/tables; JSON output is exactly one envelope. `agent` is always JSONL. No core command requires credentials. `--offline` disables all external requests, including optional AI and automatic model downloads. External downloads require an explicit separate command/configuration and approved rights.

## Request envelope

One UTF-8 JSON object per line. Version 1 is sequential: one final response per request, same order, no unsolicited progress messages on stdout. Long outputs are artifacts referenced by hash/path. Core progress goes to stderr only when requested.

```json
{"schema_version":"vmcell.protocol.v1","id":"q-001","operation":"graph.validate","arguments":{"model":"./demo/toy-scm.json"}}
```

Use strict typed arguments; reject unknown operations, versions, fields and duplicate keys. Future unavailable operations are rejected before argument/model evaluation. Schema evolution requires version negotiation; clients should tolerate documented optional response fields. IDs contain 1–128 ASCII letters, digits, underscores, periods, colons or hyphens. A malformed line may produce `id:null`. Duplicate IDs are rejected, including IDs used in a syntactically valid but invalid request.

## Scientific result envelope

Every causal success or rejection includes the same scientific context keys. When a value cannot be known, use null and a reason rather than inventing a revision. Noncausal commands use a documented `scientific_context:null`.

```json
{
  "schema_version": "vmcell.result.v1",
  "id": "q-002",
  "status": "unsupported",
  "exit_code": 3,
  "query_level": "counterfactual",
  "result": null,
  "error": {
    "code": "unsupported_operation",
    "message": "This operation is planned but not implemented. No scientific computation was performed.",
    "retryable": false
  },
  "scientific_context": {
    "model_revision": null,
    "model_hash": null,
    "model_resolution": "not_loaded: operation is not implemented",
    "basis": null,
    "identification": {"status": "unsupported", "reason": "scientific_engine_not_implemented"},
    "assumptions": [],
    "uncertainty": {"status": "not_computed", "reason": "no_scientific_computation_performed"},
    "validation_tier": "unvalidated",
    "provenance": [{"kind": "software", "source": "cyra/0.1.0:sprint-00"}]
  },
  "artifacts": [],
  "execution": {"seed": null, "duration_ms": null, "scientific_computation_performed": false}
}
```

Production outputs require real hashes for resolved model/input artifacts. `result` is null on a scientifically refused computation; return no incidental numeric effect. Identification-only commands may successfully return a certificate whose status is `not_identifiable`; distinguish executing an analysis successfully from fulfilling a rejected numeric request.

Uncertainty status is one of `quantified`, `conditional_point`, `not_quantified`, `not_applicable`, `not_computed`, with method, scope, assumptions, level and intervals/distribution when appropriate. Never substitute Jev confidence for this field.

## Exit codes

| Code | Meaning for one-shot commands |
| --- | --- |
| 0 | Requested operation completed, including an explicitly requested identification report |
| 2 | Invalid arguments, syntax, schema, units or graph |
| 3 | Unsupported scientific query/model family |
| 4 | Requested numeric effect is not identifiable under the stated basis |
| 5 | Data/license/provenance/evidence eligibility failure |
| 6 | Numerical failure, invalid posterior or convergence failure |
| 7 | Resource limit or required optional adapter unavailable |
| 8 | Filesystem/permission/integrity error |
| 70 | Unexpected internal failure |

In JSONL, recoverable per-request errors carry the equivalent `exit_code` in their response and the session continues. Process exit is 0 at clean EOF, even when individual requests failed; fatal transport/internal failures use the table. Document this distinction for agent clients. Reject malformed lines and recover at the next newline; an oversized line ends the session after a bounded error response.

## Limits and permissions

Implemented limits: 1 MiB per request line including newline, 64 container nesting levels, 4096 items per container, 4096 request lines per session and one in-flight request. A session limit ends the session with exit 7 and a final error; reopen to continue. Blank lines are malformed requests. CRLF and final lines without newline are supported. Per-operation simulation time/memory quotas will arrive with those operations. No network listener is used.

One-shot execution duration is null when not measured; JSONL reports elapsed request handling milliseconds. No scientific RNG or seed is applied. Donor splits use a documented SHA-256 bucket, not a statistical RNG. The request schema describes transport; operation-specific argument checks are enforced by the dispatcher. Human-readable `schema` prints the schema object; JSON mode wraps it in the result envelope.

## Sprint 01 measurement operations

`import` reads `--input`, `--mapping`, `--metadata` and `--out`. Exit 0 means the dataset is eligible for pipeline use. Exit 5 means a manifest was still published and the dataset is quarantined or ineligible; it is excluded from benchmark plans. Exit 2 is a validation failure and leaves no output directory. Exit 8 is an IO or integrity failure, including an import stopped before publication. A declared source checksum that does not match the input bytes is rejected and the file is not rewritten.

`observe --dataset <dir> --request <file>` retrieves stored measurements. The request kind must be `measurement_retrieval`. The response uses query level `association`, identification `not_applicable`, and `scientific_computation_performed: false`. It is not an association estimate. An empty `observe` argument object remains the unimplemented scientific operation and exits 3. `association_estimand` is rejected with exit 3 and no estimate.

`provenance --artifact <path>` resolves a dataset directory, its `manifest.json`, or a file listed by that manifest. Human and JSON output use the same lineage, license, origin and unavailable fields. No network lookup is performed.

JSONL uses the same functions. Import arguments are `input`, `mapping`, `metadata` and `out`. Observe arguments are `dataset` and `request`. Provenance takes `artifact`. Paths are strings.

Create private workspace directories with mode `0700` and sensitive files with `0600` on Unix, with a documented equivalent on supported platforms. Refuse overwrites of immutable artifacts. Imported model references must not escape their bundle through traversal or symlinks. Explicit caller-supplied input paths remain supported; untrusted references inside a file do not grant broader filesystem access. The reasoning tool API uses a narrower artifact allowlist.

Use JSON escaping for all strings, reject non-finite numerical values, and separate diagnostic stderr from result stdout. Test both TTY and piped execution, credentials in errors, broken pipes, interruption, corrupted inputs and repeat execution.
