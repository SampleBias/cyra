//! Unit identity is explicit. Different quantity kinds are never converted implicitly.
use crate::error::DataError;
use crate::model::{Modality, QuantityKind};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitDef {
    pub name: &'static str,
    pub kind: QuantityKind,
    pub modalities: &'static [Modality],
}

const UNITS: &[UnitDef] = &[
    UnitDef {
        name: "count",
        kind: QuantityKind::Count,
        modalities: &[Modality::Transcript],
    },
    UnitDef {
        name: "tpm",
        kind: QuantityKind::RelativeExpression,
        modalities: &[Modality::Transcript],
    },
    UnitDef {
        name: "fpkm",
        kind: QuantityKind::RelativeExpression,
        modalities: &[Modality::Transcript],
    },
    UnitDef {
        name: "abundance_au",
        kind: QuantityKind::Abundance,
        modalities: &[Modality::IntracellularProtein],
    },
    UnitDef {
        name: "molecules_per_cell",
        kind: QuantityKind::Abundance,
        modalities: &[Modality::IntracellularProtein],
    },
    UnitDef {
        name: "activity_au",
        kind: QuantityKind::Activity,
        modalities: &[Modality::IntracellularProtein],
    },
    UnitDef {
        name: "umol_per_min_per_mg",
        kind: QuantityKind::Activity,
        modalities: &[Modality::IntracellularProtein],
    },
    UnitDef {
        name: "pg_per_ml",
        kind: QuantityKind::Concentration,
        modalities: &[Modality::SecretedProtein],
    },
    UnitDef {
        name: "ng_per_ml",
        kind: QuantityKind::Concentration,
        modalities: &[Modality::SecretedProtein],
    },
    UnitDef {
        name: "pg",
        kind: QuantityKind::Amount,
        modalities: &[Modality::SecretedProtein],
    },
    UnitDef {
        name: "pg_per_cell_per_hour",
        kind: QuantityKind::Flux,
        modalities: &[Modality::SecretedProtein],
    },
    UnitDef {
        name: "micromol_per_l",
        kind: QuantityKind::Concentration,
        modalities: &[Modality::Metabolite],
    },
    UnitDef {
        name: "nmol_per_mg",
        kind: QuantityKind::SpecificAmount,
        modalities: &[Modality::Metabolite],
    },
    UnitDef {
        name: "percent",
        kind: QuantityKind::FunctionalReadout,
        modalities: &[Modality::FunctionalAssay],
    },
    UnitDef {
        name: "absorbance_au",
        kind: QuantityKind::FunctionalReadout,
        modalities: &[Modality::FunctionalAssay],
    },
    UnitDef {
        name: "fold_change",
        kind: QuantityKind::FunctionalReadout,
        modalities: &[Modality::FunctionalAssay],
    },
];

pub fn define(name: &str) -> Result<&'static UnitDef, DataError> {
    UNITS.iter().find(|unit| unit.name == name).ok_or_else(|| {
        DataError::invalid(format!(
            "Unrecognized unit '{name}'. Units are not inferred or rewritten without an explicit conversion in the closed vocabulary."
        ))
    })
}

pub fn compatible(unit: &str, modality: Modality) -> Result<QuantityKind, DataError> {
    let defined = define(unit)?;
    if !defined.modalities.contains(&modality) {
        return Err(DataError::invalid(format!(
            "Unit '{unit}' is not valid for modality {}. Count, expression, concentration, amount, flux and activity stay distinct.",
            modality.as_str()
        )));
    }
    Ok(defined.kind)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnitConversion {
    pub analyte_id: String,
    pub from_unit: String,
    pub to_unit: String,
    pub factor: String,
}

pub fn conversion_allowed(
    conversion: &UnitConversion,
    modality: Modality,
) -> Result<(), DataError> {
    let from = define(&conversion.from_unit)?;
    let to = define(&conversion.to_unit)?;
    if from.kind != to.kind
        || !from.modalities.contains(&modality)
        || !to.modalities.contains(&modality)
    {
        return Err(DataError::invalid(format!(
            "Conversion from {} to {} is rejected because the quantity kind or modality differs. No unit was repaired.",
            conversion.from_unit, conversion.to_unit
        )));
    }
    if crate::decimal::canonical_decimal(&conversion.factor).is_err() {
        return Err(DataError::invalid(
            "A conversion factor must be a plain positive decimal.",
        ));
    }
    let factor = crate::decimal::canonical_decimal(&conversion.factor)?;
    if factor.starts_with('-') || factor == "0" {
        return Err(DataError::invalid(
            "A conversion factor must be a positive decimal.",
        ));
    }
    Ok(())
}
