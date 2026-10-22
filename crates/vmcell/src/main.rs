use clap::{Parser, Subcommand, ValueEnum};
use cyra_core::protocol::{ErrorCode, Operation, Response};
use cyra_core::scientific::{
    Identification, IdentificationStatus, InferenceBasis, ProvenanceReference, ScientificContext,
    Uncertainty, UncertaintyStatus, ValidationTier,
};
use cyra_core::{VERSION, protocol, service};
use cyra_data::{self, ImportRequest};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Format {
    Human,
    Json,
}

#[derive(Parser)]
#[command(
    name = "vmcell",
    version,
    about = "Cyra — Cybernetic Cellular Intelligence. Measurement ingestion is available; scientific simulation is not yet implemented."
)]
struct Cli {
    #[arg(long, global = true, value_enum, default_value = "human")]
    format: Format,
    /// Prohibit all network access. This foundation has no network adapters.
    #[arg(long, global = true)]
    offline: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Report project, build policy and schema versions.
    Version,
    /// Discover implemented and planned capabilities.
    Capabilities,
    /// Return a versioned JSON Schema inside the result envelope.
    Schema { name: String },
    /// Read one JSON request per line and write one response per line.
    Agent {
        #[arg(long,default_value="jsonl",value_parser=["jsonl"])]
        protocol: String,
    },
    /// Validate and ingest a CSV, TSV or JSON measurement table.
    Import {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        mapping: PathBuf,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Retrieve stored measurements. This does not estimate an association.
    Observe {
        #[arg(long)]
        dataset: PathBuf,
        #[arg(long)]
        request: PathBuf,
    },
    /// Resolve local lineage, license and origin for a dataset artifact.
    Provenance {
        #[arg(long)]
        artifact: PathBuf,
    },
    #[command(external_subcommand)]
    Other(Vec<OsString>),
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            if error.kind() != io::ErrorKind::BrokenPipe {
                eprintln!("vmcell: input/output failure");
            }
            ExitCode::from(8)
        }
    }
}

fn run() -> io::Result<u8> {
    let raw: Vec<OsString> = std::env::args_os().collect();
    let wants_json = raw.iter().any(|s| s == "--format=json" || s == "agent")
        || raw.windows(2).any(|w| w[0] == "--format" && w[1] == "json");
    let cli = match Cli::try_parse_from(&raw) {
        Ok(cli) => cli,
        Err(error) => {
            let informational = matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            );
            let response = if informational {
                Response::success(None, json!({"text":error.to_string()}))
            } else {
                Response::failure(
                    None,
                    ErrorCode::InvalidArguments,
                    "Invalid command or arguments. Run vmcell --help for usage.",
                )
            };
            return render(
                &response,
                if wants_json {
                    Format::Json
                } else {
                    Format::Human
                },
            );
        }
    };
    // Both modes call the same service functions. There are no remote providers.
    let response = match cli.command {
        Command::Agent { .. } => {
            return protocol::serve_with(
                &mut io::stdin().lock(),
                &mut io::stdout().lock(),
                &mut |id, operation, arguments| dispatch(id, operation, arguments),
            );
        }
        Command::Version => dispatch(None, Operation::Version, json!({})),
        Command::Capabilities => dispatch(None, Operation::Capabilities, json!({})),
        Command::Schema { name } => dispatch(None, Operation::Schema, json!({"name": name})),
        Command::Import {
            input,
            mapping,
            metadata,
            out,
        } => dispatch(
            None,
            Operation::Import,
            json!({"input": input, "mapping": mapping, "metadata": metadata, "out": out}),
        ),
        Command::Observe { dataset, request } => dispatch(
            None,
            Operation::Observe,
            json!({"dataset": dataset, "request": request}),
        ),
        Command::Provenance { artifact } => {
            dispatch(None, Operation::Provenance, json!({"artifact": artifact}))
        }
        Command::Other(args) => {
            let name = args.first().and_then(|s| s.to_str()).unwrap_or("");
            let operation = match name {
                "graph" => match args.get(1).and_then(|s| s.to_str()) {
                    Some("inspect") => Some(Operation::GraphInspect),
                    Some("validate") => Some(Operation::GraphValidate),
                    Some("identify") => Some(Operation::GraphIdentify),
                    _ => None,
                },
                "model" => match args.get(1).and_then(|s| s.to_str()) {
                    Some("propose") => Some(Operation::ModelPropose),
                    Some("review") => Some(Operation::ModelReview),
                    Some("approve") => Some(Operation::ModelApprove),
                    _ => None,
                },
                other => serde_json::from_value::<Operation>(json!(other)).ok(),
            };
            match operation {
                Some(operation) => Response::unavailable(None, operation),
                None => Response::failure(
                    None,
                    ErrorCode::InvalidArguments,
                    "Unknown command. Run vmcell --help or vmcell capabilities.",
                ),
            }
        }
    };
    render(
        &response,
        if wants_json { Format::Json } else { cli.format },
    )
}

fn render(response: &Response, format: Format) -> io::Result<u8> {
    let mut output = io::stdout().lock();
    if format == Format::Json {
        protocol::write_response(&mut output, response)?;
    } else if let Some(text) = response
        .result()
        .and_then(|value| value.get("text"))
        .and_then(Value::as_str)
    {
        writeln!(output, "{text}")?;
        if response.exit_code() != 0 {
            writeln!(output, "(exit {})", response.exit_code())?;
        }
    } else if let Some(error) = response.error() {
        writeln!(output, "{} (exit {})", error.message, response.exit_code())?;
    } else if let Some(value) = response.result() {
        if let Some(text) = value.get("text").and_then(|v| v.as_str()) {
            write!(output, "{text}")?;
        } else if value.get("project").is_some() {
            writeln!(
                output,
                "Cyra — Cybernetic Cellular Intelligence\nvmcell {} · Sprint 00 foundation; Sprint 01 measurement ingestion\nNo biological simulator or validated physiological model is available yet.",
                cyra_core::VERSION
            )?;
        } else {
            serde_json::to_writer_pretty(&mut output, value).map_err(io::Error::other)?;
            writeln!(output)?;
        }
    }
    output.flush()?;
    Ok(response.exit_code())
}

fn dispatch(
    id: Option<cyra_core::scientific::Identifier>,
    operation: Operation,
    arguments: Value,
) -> Response {
    match operation {
        Operation::Version => service::dispatch(id, operation, arguments).map_result(|value| {
            value["stage"] = json!("sprint_01_data_provenance");
            value["foundation"] = json!("sprint_00");
            value["data_ingestion"] = json!("csv_tsv_json");
            value["simulator"] = json!("not_implemented");
        }),
        Operation::Capabilities => service::dispatch(id, operation, arguments).map_result(|value| {
            value["available_operations"] = json!(["version", "capabilities", "schema", "import", "observe", "provenance"]);
            value["planned_operations"] = json!([
                "init", "graph.inspect", "graph.validate", "graph.identify", "simulate", "intervene",
                "counterfactual", "explain", "reproduce", "route", "reason", "model.propose",
                "model.review", "model.approve", "fit", "evaluate"
            ]);
            value["schemas"] = json!([
                "request", "result", "scientific-query", "artifact-reference",
                "import-mapping", "dataset-manifest", "observation-request", "measurement-record"
            ]);
            value["observe_scope"] = json!("measurement_retrieval_only");
            value["storage_profiles"] = json!({"parquet": "cyra.parquet.v1", "zarr": "cyra.zarr.v1", "compression": "uncompressed"});
            value["network_adapters"] = json!([]);
        }),
        Operation::Schema => {
            let name = arguments.get("name").and_then(Value::as_str).unwrap_or("");
            if let Some(schema) = cyra_data::schema(name) {
                Response::success(id, schema)
            } else {
                service::dispatch(id, operation, arguments)
            }
        }
        Operation::Import => match paths(&arguments, &["input", "mapping", "metadata", "out"]) {
            Ok(paths) => import_response(id, cyra_data::import_dataset(&ImportRequest {
                input: paths[0].clone(),
                mapping: paths[1].clone(),
                metadata: paths[2].clone(),
                out: paths[3].clone(),
                publish: true,
            })),
            Err(_) => Response::failure(id, ErrorCode::InvalidArguments, "Import requires input, mapping, metadata and out paths."),
        },
        Operation::Provenance => match paths(&arguments, &["artifact"]) {
            Ok(paths) => match cyra_data::provenance(&paths[0]) {
                Ok(mut value) => {
                    let text = value.get("human").and_then(Value::as_str).unwrap_or("").to_owned();
                    value["text"] = json!(text);
                    Response::success(id, value)
                }
                Err(error) => data_error(id, operation, error),
            },
            Err(_) => Response::failure(id, ErrorCode::InvalidArguments, "Provenance requires an artifact path."),
        },
        Operation::Observe => {
            if arguments.as_object().is_some_and(|object| object.is_empty()) {
                return service::dispatch(id, operation, arguments);
            }
            match paths(&arguments, &["dataset", "request"]) {
                Ok(paths) => match cyra_data::observe_dataset(&paths[0], &paths[1]) {
                    Ok((loaded, records)) => {
                        let text = format!(
                            "Retrieved {} stored measurement(s) from dataset {}.\nUnavailable fields: {}.\nThis is not an association estimate and no physiological claim is made.",
                            records.len(),
                            loaded.manifest.dataset_id,
                            loaded.manifest.unavailable_fields.join(", ")
                        );
                        Response::success(id, json!({
                            "kind": "measurement_retrieval",
                            "dataset_id": loaded.manifest.dataset_id,
                            "records": records,
                            "unavailable_fields": loaded.manifest.unavailable_fields,
                            "record_count": records.len(),
                            "eligibility": loaded.manifest.eligibility,
                            "origin": loaded.manifest.origin,
                            "text": text
                        })).with_retrieval_context(retrieval_context(&loaded.manifest_sha256))
                    }
                    Err(error) => data_error(id, operation, error),
                },
                Err(_) => Response::failure(id, ErrorCode::InvalidArguments, "Measurement retrieval requires dataset and request paths."),
            }
        }
        other => service::dispatch(id, other, arguments),
    }
}

fn paths(value: &Value, names: &[&str]) -> Result<Vec<PathBuf>, ()> {
    let Some(object) = value.as_object() else {
        return Err(());
    };
    if object.len() != names.len() || names.iter().any(|name| !object.contains_key(*name)) {
        return Err(());
    }
    names
        .iter()
        .map(|name| {
            object
                .get(*name)
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .ok_or(())
        })
        .collect()
}

fn import_response(
    id: Option<cyra_core::scientific::Identifier>,
    outcome: Result<cyra_data::ImportOutcome, cyra_data::DataError>,
) -> Response {
    match outcome {
        Ok(outcome) => {
            let mut value = outcome.summary;
            value["text"] = json!(outcome.human);
            if outcome.exit_code == 0 {
                Response::success(id, value)
            } else {
                Response::eligibility(
                    id,
                    "Dataset was recorded but is not eligible. It is excluded from benchmark plans.",
                    value,
                )
            }
        }
        Err(error) => data_error(id, Operation::Import, error),
    }
}

fn data_error(
    id: Option<cyra_core::scientific::Identifier>,
    operation: Operation,
    error: cyra_data::DataError,
) -> Response {
    let code = match error.exit_code {
        3 => ErrorCode::UnsupportedOperation,
        5 => ErrorCode::IneligibleData,
        8 => ErrorCode::IoFailure,
        _ => ErrorCode::InvalidArguments,
    };
    let response = Response::failure(id, code, &error.message);
    if code == ErrorCode::UnsupportedOperation {
        response.mark_scientific(operation)
    } else {
        response
    }
}

fn retrieval_context(dataset_sha: &str) -> ScientificContext {
    ScientificContext {
        model_revision: None,
        model_hash: None,
        model_resolution: "not_a_model: stored measurements were retrieved".to_owned(),
        basis: Some(InferenceBasis::ObservedData),
        identification: Identification {
            status: IdentificationStatus::NotApplicable,
            reason: "measurement_retrieval_is_not_an_estimand".to_owned(),
        },
        assumptions: vec![
            "No imputation, unit repair or causal adjustment was applied.".to_owned(),
        ],
        uncertainty: Uncertainty {
            status: UncertaintyStatus::NotApplicable,
            reason: "stored measurements do not by themselves form a computed interval".to_owned(),
        },
        validation_tier: ValidationTier::Unvalidated,
        provenance: vec![
            ProvenanceReference {
                kind: "software".to_owned(),
                source: format!("cyra/{VERSION}:sprint-01"),
            },
            ProvenanceReference {
                kind: "dataset_manifest".to_owned(),
                source: dataset_sha.to_owned(),
            },
        ],
    }
}
