use crate::scientific::{Identifier, QueryLevel, ScientificContext};
use crate::strict_json::{self, MAX_LINE_BYTES};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};
use std::time::Instant;

pub const MAX_SESSION_REQUESTS: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProtocolVersion {
    #[serde(rename = "vmcell.protocol.v1")]
    V1,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ResultVersion {
    #[serde(rename = "vmcell.result.v1")]
    V1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum Operation {
    #[serde(rename = "version")]
    Version,
    #[serde(rename = "capabilities")]
    Capabilities,
    #[serde(rename = "schema")]
    Schema,
    #[serde(rename = "init")]
    Init,
    #[serde(rename = "import")]
    Import,
    #[serde(rename = "graph.inspect")]
    GraphInspect,
    #[serde(rename = "graph.validate")]
    GraphValidate,
    #[serde(rename = "graph.identify")]
    GraphIdentify,
    #[serde(rename = "observe")]
    Observe,
    #[serde(rename = "simulate")]
    Simulate,
    #[serde(rename = "intervene")]
    Intervene,
    #[serde(rename = "counterfactual")]
    Counterfactual,
    #[serde(rename = "explain")]
    Explain,
    #[serde(rename = "provenance")]
    Provenance,
    #[serde(rename = "reproduce")]
    Reproduce,
    #[serde(rename = "route")]
    Route,
    #[serde(rename = "reason")]
    Reason,
    #[serde(rename = "model.propose")]
    ModelPropose,
    #[serde(rename = "model.review")]
    ModelReview,
    #[serde(rename = "model.approve")]
    ModelApprove,
    #[serde(rename = "fit")]
    Fit,
    #[serde(rename = "evaluate")]
    Evaluate,
}
impl Operation {
    pub fn query_level(self) -> Option<QueryLevel> {
        match self {
            Self::Observe => Some(QueryLevel::Association),
            Self::Intervene | Self::GraphIdentify => Some(QueryLevel::Intervention),
            Self::Counterfactual => Some(QueryLevel::Counterfactual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: ProtocolVersion,
    pub id: Identifier,
    pub operation: Operation,
    pub arguments: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Success,
    InvalidInput,
    Unsupported,
    NotIdentifiable,
    DataError,
    NumericalFailure,
    ResourceLimit,
    IoError,
    InternalError,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidJson,
    InvalidRequest,
    InvalidArguments,
    UnknownSchema,
    DuplicateRequestId,
    UnsupportedOperation,
    NonIdentifiable,
    IneligibleData,
    NumericalFailure,
    RequestTooLarge,
    SessionLimit,
    AdapterUnavailable,
    IoFailure,
    InternalFailure,
}
impl ErrorCode {
    pub fn exit_code(self) -> u8 {
        match self {
            Self::InvalidJson
            | Self::InvalidRequest
            | Self::InvalidArguments
            | Self::UnknownSchema
            | Self::DuplicateRequestId => 2,
            Self::UnsupportedOperation => 3,
            Self::NonIdentifiable => 4,
            Self::IneligibleData => 5,
            Self::NumericalFailure => 6,
            Self::RequestTooLarge | Self::SessionLimit | Self::AdapterUnavailable => 7,
            Self::IoFailure => 8,
            Self::InternalFailure => 70,
        }
    }
    fn status(self) -> Status {
        match self.exit_code() {
            2 => Status::InvalidInput,
            3 => Status::Unsupported,
            4 => Status::NotIdentifiable,
            5 => Status::DataError,
            6 => Status::NumericalFailure,
            7 => Status::ResourceLimit,
            8 => Status::IoError,
            _ => Status::InternalError,
        }
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    pub seed: Option<u64>,
    pub duration_ms: Option<u64>,
    pub scientific_computation_performed: bool,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Response {
    schema_version: ResultVersion,
    id: Option<Identifier>,
    status: Status,
    exit_code: u8,
    query_level: Option<QueryLevel>,
    result: Option<Value>,
    error: Option<ErrorDetail>,
    scientific_context: Option<ScientificContext>,
    artifacts: Vec<crate::artifact::ArtifactReference>,
    execution: Execution,
}
impl Response {
    pub fn success(id: Option<Identifier>, value: Value) -> Self {
        Self {
            schema_version: ResultVersion::V1,
            id,
            status: Status::Success,
            exit_code: 0,
            query_level: None,
            result: Some(value),
            error: None,
            scientific_context: None,
            artifacts: vec![],
            execution: Execution {
                seed: None,
                duration_ms: None,
                scientific_computation_performed: false,
            },
        }
    }
    pub fn failure(id: Option<Identifier>, code: ErrorCode, message: &str) -> Self {
        Self {
            schema_version: ResultVersion::V1,
            id,
            status: code.status(),
            exit_code: code.exit_code(),
            query_level: None,
            result: None,
            error: Some(ErrorDetail {
                code,
                message: message.into(),
                retryable: false,
            }),
            scientific_context: None,
            artifacts: vec![],
            execution: Execution {
                seed: None,
                duration_ms: None,
                scientific_computation_performed: false,
            },
        }
    }
    pub fn unavailable(id: Option<Identifier>, operation: Operation) -> Self {
        let mut response = Self::failure(
            id,
            ErrorCode::UnsupportedOperation,
            "This operation is planned but not implemented. No scientific computation was performed.",
        );
        response.query_level = operation.query_level();
        if response.query_level.is_some() {
            response.scientific_context = Some(ScientificContext::unavailable());
        }
        response
    }
    /// Eligibility was decided and recorded. This is not a scientific estimate.
    pub fn eligibility(id: Option<Identifier>, message: &str, result: Value) -> Self {
        Self {
            schema_version: ResultVersion::V1,
            id,
            status: Status::DataError,
            exit_code: ErrorCode::IneligibleData.exit_code(),
            query_level: None,
            result: Some(result),
            error: Some(ErrorDetail {
                code: ErrorCode::IneligibleData,
                message: message.into(),
                retryable: false,
            }),
            scientific_context: None,
            artifacts: vec![],
            execution: Execution {
                seed: None,
                duration_ms: None,
                scientific_computation_performed: false,
            },
        }
    }
    pub fn mark_scientific(mut self, operation: Operation) -> Self {
        self.query_level = operation.query_level();
        if self.query_level.is_some() && self.scientific_context.is_none() {
            self.scientific_context = Some(ScientificContext::unavailable());
        }
        self
    }
    pub fn with_retrieval_context(mut self, context: ScientificContext) -> Self {
        self.query_level = Some(QueryLevel::Association);
        self.scientific_context = Some(context);
        self
    }
    pub fn map_result(mut self, update: impl FnOnce(&mut Value)) -> Self {
        if let Some(value) = self.result.as_mut() {
            update(value);
        }
        self
    }
    pub fn exit_code(&self) -> u8 {
        self.exit_code
    }
    pub fn result(&self) -> Option<&Value> {
        self.result.as_ref()
    }
    pub fn error(&self) -> Option<&ErrorDetail> {
        self.error.as_ref()
    }
}

/// Reads at most limit+1 bytes, including the newline. Never drains an oversized line.
fn read_line_bounded(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            return Ok(if line.is_empty() { None } else { Some(line) });
        }
        let remaining = MAX_LINE_BYTES + 1 - line.len();
        let count = buffer
            .iter()
            .position(|b| *b == b'\n')
            .map_or(buffer.len(), |i| i + 1)
            .min(remaining);
        line.extend_from_slice(&buffer[..count]);
        reader.consume(count);
        if line.len() > MAX_LINE_BYTES || line.last() == Some(&b'\n') {
            return Ok(Some(line));
        }
    }
}

pub fn write_response(writer: &mut impl Write, response: &Response) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, response).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Returns a process exit code. Individual request failures do not fail a clean session.
pub fn serve(reader: &mut impl BufRead, writer: &mut impl Write) -> io::Result<u8> {
    serve_with(reader, writer, &mut |id, operation, arguments| {
        crate::service::dispatch(id, operation, arguments)
    })
}

/// Same session rules as [`serve`], with a caller-supplied operation dispatcher.
pub fn serve_with(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    dispatch: &mut dyn FnMut(Option<Identifier>, Operation, Value) -> Response,
) -> io::Result<u8> {
    let mut ids = BTreeSet::new();
    let mut requests = 0usize;
    while let Some(line) = read_line_bounded(reader)? {
        if line.len() > MAX_LINE_BYTES {
            write_response(
                writer,
                &Response::failure(
                    None,
                    ErrorCode::RequestTooLarge,
                    "Request line exceeds the byte limit; session terminated.",
                ),
            )?;
            return Ok(7);
        }
        if requests >= MAX_SESSION_REQUESTS {
            write_response(
                writer,
                &Response::failure(
                    None,
                    ErrorCode::SessionLimit,
                    "Session request limit reached; start a new session.",
                ),
            )?;
            return Ok(7);
        }
        requests += 1;
        let start = Instant::now();
        let mut response = handle_line(&line, &mut ids, dispatch);
        response.execution.duration_ms =
            Some(start.elapsed().as_millis().min(u64::MAX as u128) as u64);
        write_response(writer, &response)?;
    }
    Ok(0)
}

fn handle_line(
    line: &[u8],
    ids: &mut BTreeSet<Identifier>,
    dispatch: &mut dyn FnMut(Option<Identifier>, Operation, Value) -> Response,
) -> Response {
    let value = match strict_json::parse(line) {
        Ok(v) => v,
        Err(_) => {
            return Response::failure(
                None,
                ErrorCode::InvalidJson,
                "Invalid JSON, duplicate keys, or structural limit exceeded.",
            );
        }
    };
    // Error messages deliberately never echo the payload (which may contain credentials).
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .and_then(|s| Identifier::try_from(s.to_owned()).ok());
    if let Some(ref id) = id
        && !ids.insert(id.clone())
    {
        return Response::failure(
            Some(id.clone()),
            ErrorCode::DuplicateRequestId,
            "Request ID has already been used in this session.",
        );
    }
    let request = match serde_json::from_value::<Request>(value) {
        Ok(request) => request,
        Err(_) => {
            return Response::failure(
                id,
                ErrorCode::InvalidRequest,
                "Request does not match vmcell.protocol.v1; check schema, operation, ID, and fields.",
            );
        }
    };
    dispatch(Some(request.id), request.operation, request.arguments)
}
