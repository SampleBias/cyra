//! Shared service entry points for the terminal and agent protocol.
use crate::{
    protocol::{ErrorCode, Operation, Request, Response},
    scientific::{Identifier, ScientificQuery},
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyArguments {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchemaArguments {
    name: String,
}

pub fn dispatch(id: Option<Identifier>, operation: Operation, arguments: Value) -> Response {
    if matches!(operation, Operation::Version | Operation::Capabilities)
        && serde_json::from_value::<EmptyArguments>(arguments.clone()).is_err()
    {
        return Response::failure(
            id,
            ErrorCode::InvalidArguments,
            "This operation requires an empty arguments object.",
        );
    }
    match operation {
        Operation::Version => Response::success(
            id,
            json!({
                "project": crate::PROJECT_NAME, "expansion": crate::PROJECT_EXPANSION,
                "version": crate::VERSION, "cli": "vmcell", "stage": "sprint_00_foundation",
                "protocol": "vmcell.protocol.v1", "result_schema": "vmcell.result.v1",
            "toolchain": env!("CYRA_BUILD_RUSTC"), "pinned_toolchain": "1.97.1",
            "target": env!("CYRA_BUILD_TARGET"), "build_profile": env!("CYRA_BUILD_PROFILE"),
            "features": [], "scientific_validation": "unvalidated"
            }),
        ),
        Operation::Capabilities => Response::success(
            id,
            json!({
                "available_operations": ["version","capabilities","schema"],
                "transport": ["cli", "jsonl"],
                "planned_operations": ["init","import","graph.inspect","graph.validate","graph.identify","observe","simulate","intervene","counterfactual","explain","provenance","reproduce","route","reason","model.propose","model.review","model.approve","fit","evaluate"],
                "scientific_model_families": [], "offline": true, "network_adapters": [],
                "schemas": ["request","result","scientific-query","artifact-reference"],
                "limits": {"line_bytes_including_newline": crate::strict_json::MAX_LINE_BYTES, "json_depth": crate::strict_json::MAX_DEPTH, "container_items": crate::strict_json::MAX_CONTAINER_ITEMS, "session_requests": crate::protocol::MAX_SESSION_REQUESTS, "request_id_bytes":128},
                "supported_reference_target":"x86_64-unknown-linux-gnu"
            }),
        ),
        Operation::Schema => {
            let Ok(args) = serde_json::from_value::<SchemaArguments>(arguments) else {
                return Response::failure(
                    id,
                    ErrorCode::InvalidArguments,
                    "Schema requires exactly one string field: name.",
                );
            };
            match schema(&args.name) {
                Some(value) => Response::success(id, value),
                None => Response::failure(
                    id,
                    ErrorCode::UnknownSchema,
                    "Unknown schema name; see capabilities for available schemas.",
                ),
            }
        }
        _ => Response::unavailable(id, operation),
    }
}

pub fn schema(name: &str) -> Option<Value> {
    let schema = match name {
        "request" => schemars::schema_for!(Request),
        "result" => schemars::schema_for!(Response),
        "scientific-query" => schemars::schema_for!(ScientificQuery),
        "artifact-reference" => schemars::schema_for!(crate::artifact::ArtifactReference),
        _ => return None,
    };
    serde_json::to_value(schema).ok()
}
