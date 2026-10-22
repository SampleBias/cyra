use cyra_core::{
    protocol::{self, ErrorCode, MAX_SESSION_REQUESTS, Response},
    scientific::{FiniteValue, Identifier, ScientificQuery},
    strict_json,
};
use serde_json::{Value, json};
use std::io::Cursor;

fn run(input: &[u8]) -> (u8, Vec<Value>) {
    let mut out = Vec::new();
    let exit = protocol::serve(&mut Cursor::new(input), &mut out).unwrap();
    let responses = String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (exit, responses)
}
fn request(id: &str, operation: &str, args: Value) -> String {
    format!(
        "{}\n",
        json!({"schema_version":"vmcell.protocol.v1","id":id,"operation":operation,"arguments":args})
    )
}

#[test]
fn a00_03_recovers_after_malformed_json_and_preserves_order() {
    let input = format!(
        "{}{{broken\n{}",
        request("a", "version", json!({})),
        request("b", "capabilities", json!({}))
    );
    let (exit, r) = run(input.as_bytes());
    assert_eq!(exit, 0);
    assert_eq!(r.len(), 3);
    assert_eq!(r[0]["id"], "a");
    assert_eq!(r[1]["exit_code"], 2);
    assert_eq!(r[2]["id"], "b");
    assert_eq!(r[0]["result"]["project"], "Cyra");
}

#[test]
fn a00_04_rejects_duplicate_keys_at_every_depth_including_escaped_aliases() {
    for input in [
        r#"{"id":"a","id":"b"}"#,
        r#"{"x":{"a":1,"\u0061":2}}"#,
        r#"{"x":[{"a":1,"a":2}]}"#,
    ] {
        assert!(strict_json::parse(input.as_bytes()).is_err());
    }
}

#[test]
fn a00_04_depth_size_container_and_nonfinite_limits() {
    let allowed = format!("{}0{}", "[".repeat(64), "]".repeat(64));
    let rejected = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    assert!(strict_json::parse(allowed.as_bytes()).is_ok());
    assert!(strict_json::parse(rejected.as_bytes()).is_err());
    assert!(strict_json::parse(&vec![b' '; strict_json::MAX_LINE_BYTES + 1]).is_err());
    assert!(
        strict_json::parse(
            format!("[{}0]", "0,".repeat(strict_json::MAX_CONTAINER_ITEMS)).as_bytes()
        )
        .is_err()
    );
    for input in ["NaN", "Infinity", "1e999", "{} {}"] {
        assert!(strict_json::parse(input.as_bytes()).is_err());
    }
    assert!(FiniteValue::try_from(f64::NAN).is_err());
    assert!(FiniteValue::try_from(f64::INFINITY).is_err());
}

#[test]
fn a00_04_unknown_schema_operation_and_fields_are_rejected_without_echo() {
    let secret = "secret_should_never_be_echoed";
    let values = [
        json!({"schema_version":"vmcell.protocol.v2","id":"a","operation":"version","arguments":{}}),
        json!({"schema_version":"vmcell.protocol.v1","id":"b","operation":secret,"arguments":{}}),
        json!({"schema_version":"vmcell.protocol.v1","id":"c","operation":"version","arguments":{},"token":secret}),
        json!({"schema_version":"vmcell.protocol.v1","id":"d","operation":"version","arguments":{"token":secret}}),
        json!({"schema_version":"vmcell.protocol.v1","id":"e","operation":"schema","arguments":{"name":"request","extra":1}}),
    ];
    for value in values {
        let (_, r) = run(value.to_string().as_bytes());
        assert_eq!(r[0]["exit_code"], 2);
        assert!(r[0]["result"].is_null());
        assert!(!r[0].to_string().contains(secret));
    }
}

#[test]
fn a00_04_duplicate_ids_and_session_growth_are_bounded() {
    let (exit, r) = run(format!(
        "{}{}",
        request("same", "version", json!({})),
        request("same", "capabilities", json!({}))
    )
    .as_bytes());
    assert_eq!(exit, 0);
    assert_eq!(r[1]["error"]["code"], "duplicate_request_id");
    let input = (0..MAX_SESSION_REQUESTS + 1)
        .map(|i| request(&format!("r{i}"), "version", json!({})))
        .collect::<String>();
    let (exit, r) = run(input.as_bytes());
    assert_eq!(exit, 7);
    assert_eq!(r.len(), MAX_SESSION_REQUESTS + 1);
    assert_eq!(r.last().unwrap()["error"]["code"], "session_limit");
}

#[test]
fn a00_04_oversized_line_terminates_without_processing_following_request() {
    let mut input = vec![b'x'; strict_json::MAX_LINE_BYTES + 1];
    input.extend_from_slice(b"\n");
    input.extend_from_slice(request("later", "version", json!({})).as_bytes());
    let mut cursor = Cursor::new(input);
    let mut out = Vec::new();
    assert_eq!(protocol::serve(&mut cursor, &mut out).unwrap(), 7);
    assert_eq!(cursor.position() as usize, strict_json::MAX_LINE_BYTES + 1);
    let r: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(r["error"]["code"], "request_too_large");
}

#[test]
fn a00_05_crlf_eof_utf8_and_blank_line_behavior() {
    let input = format!(
        "\n{}",
        request("ok", "version", json!({})).replace('\n', "\r\n")
    );
    let (_, r) = run(input.as_bytes());
    assert_eq!(r.len(), 2);
    assert_eq!(r[1]["status"], "success");
    let (_, r) = run(request("eof", "version", json!({})).trim_end().as_bytes());
    assert_eq!(r.len(), 1);
    let (_, r) = run(&[0xff, b'\n']);
    assert_eq!(r[0]["error"]["code"], "invalid_json");
    assert_eq!(run(b"").1.len(), 0);
}

#[test]
fn a00_03_causal_refusals_have_context_and_no_invented_results() {
    for (op, level) in [
        ("observe", "association"),
        ("intervene", "intervention"),
        ("counterfactual", "counterfactual"),
    ] {
        let (_, r) = run(request("q", op, json!({})).as_bytes());
        let r = &r[0];
        assert_eq!(r["status"], "unsupported");
        assert_eq!(r["exit_code"], 3);
        assert!(r["result"].is_null());
        assert_eq!(r["query_level"], level);
        let c = &r["scientific_context"];
        assert!(c.get("model_revision").unwrap().is_null());
        assert!(c["assumptions"].is_array());
        assert!(!c["provenance"].as_array().unwrap().is_empty());
        assert_eq!(c["uncertainty"]["status"], "not_computed");
        assert_eq!(c["identification"]["status"], "unsupported");
        assert_eq!(r["execution"]["scientific_computation_performed"], false);
    }
}

#[test]
fn a00_03_causal_types_cannot_substitute_conditioning_for_action() {
    let scope = json!({"model_revision":"m1","population":"p1","outcome":"Y"});
    let association = json!({"level":"association","scope":scope,"given":{"X":1.0}});
    assert!(serde_json::from_value::<ScientificQuery>(association.clone()).is_ok());
    let mut invalid = association;
    invalid["level"] = json!("counterfactual");
    assert!(serde_json::from_value::<ScientificQuery>(invalid).is_err());
    assert!(Identifier::try_from("../secret".to_owned()).is_err());
    assert!(Identifier::try_from("x".repeat(129)).is_err());
}

#[test]
fn a00_04_exit_code_policy_is_stable() {
    for (error, exit) in [
        (ErrorCode::InvalidRequest, 2),
        (ErrorCode::UnsupportedOperation, 3),
        (ErrorCode::NonIdentifiable, 4),
        (ErrorCode::IneligibleData, 5),
        (ErrorCode::NumericalFailure, 6),
        (ErrorCode::AdapterUnavailable, 7),
        (ErrorCode::IoFailure, 8),
        (ErrorCode::InternalFailure, 70),
    ] {
        assert_eq!(
            Response::failure(None, error, "fixed message").exit_code(),
            exit
        );
    }
}

#[test]
fn schemas_are_emitted_and_match_checked_in_contracts() {
    for name in [
        "request",
        "result",
        "scientific-query",
        "artifact-reference",
    ] {
        let current = cyra_core::service::schema(name).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../schemas/{name}.v1.json"));
        let committed: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(
            current, committed,
            "schema drift; regenerate deliberately and review compatibility"
        );
        assert_eq!(
            current["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
    }
}
