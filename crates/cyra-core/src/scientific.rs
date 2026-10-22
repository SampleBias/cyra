//! Reserved mathematical types keep conditioning separate from intervention.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Identifier(String);

impl JsonSchema for Identifier {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Identifier".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9_.:-]+$"})
    }
}

impl TryFrom<String> for Identifier {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
        {
            return Err("identifier must contain 1..128 ASCII letters, digits, or _ . : -");
        }
        Ok(Self(value))
    }
}
impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.0
    }
}
impl Identifier {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "f64", into = "f64")]
pub struct FiniteValue(f64);
impl TryFrom<f64> for FiniteValue {
    type Error = &'static str;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err("value must be finite")
        }
    }
}
impl From<FiniteValue> for f64 {
    fn from(value: FiniteValue) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryLevel {
    Association,
    Intervention,
    Counterfactual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InferenceBasis {
    ObservedData,
    ObservationalIdentification,
    SpecifiedScm,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryScope {
    pub model_revision: Identifier,
    pub population: Identifier,
    pub outcome: Identifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "level", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScientificQuery {
    Association {
        scope: QueryScope,
        given: BTreeMap<Identifier, FiniteValue>,
    },
    Intervention {
        scope: QueryScope,
        basis: InferenceBasis,
        interventions: BTreeMap<Identifier, FiniteValue>,
    },
    Counterfactual {
        scope: QueryScope,
        evidence: BTreeMap<Identifier, FiniteValue>,
        interventions: BTreeMap<Identifier, FiniteValue>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParameterOrigin {
    Illustrative,
    LiteratureDerived,
    Fitted,
    FixedMathematicalConstant,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementOrigin {
    Experimental,
    SyntheticFixture,
    Simulated,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidationTier {
    Unvalidated,
    SyntheticVerified,
    InternallyEvaluated,
    IndependentlyEvaluated,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    IllustrativeOnly,
    Unsupported,
    SupportedWithinScope,
    Inconclusive,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdentificationStatus {
    Identified,
    NotIdentifiable,
    Unsupported,
    NotApplicable,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyStatus {
    Quantified,
    ConditionalPoint,
    NotQuantified,
    NotApplicable,
    NotComputed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceReference {
    pub kind: String,
    pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Identification {
    pub status: IdentificationStatus,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Uncertainty {
    pub status: UncertaintyStatus,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScientificContext {
    pub model_revision: Option<Identifier>,
    pub model_hash: Option<String>,
    pub model_resolution: String,
    pub basis: Option<InferenceBasis>,
    pub identification: Identification,
    pub assumptions: Vec<String>,
    pub uncertainty: Uncertainty,
    pub validation_tier: ValidationTier,
    pub provenance: Vec<ProvenanceReference>,
}
impl ScientificContext {
    pub fn unavailable() -> Self {
        Self {
            model_revision: None,
            model_hash: None,
            model_resolution: "not_loaded: operation is not implemented".into(),
            basis: None,
            identification: Identification {
                status: IdentificationStatus::Unsupported,
                reason: "scientific_engine_not_implemented".into(),
            },
            assumptions: vec![],
            uncertainty: Uncertainty {
                status: UncertaintyStatus::NotComputed,
                reason: "no_scientific_computation_performed".into(),
            },
            validation_tier: ValidationTier::Unvalidated,
            provenance: vec![ProvenanceReference {
                kind: "software".into(),
                source: format!("cyra/{}:sprint-00", crate::VERSION),
            }],
        }
    }
}
