use crate::transport::{
    dto::{PhaseCode, PhaseDefinitionDto},
    error::{AppError, ErrorCode},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const PRE_V1: &str = include_str!("../../../../docs/architecture/phase-definitions/PRE.v1.json");
const P1_V1: &str = include_str!("../../../../docs/architecture/phase-definitions/P1.v1.json");
const P2_V1: &str = include_str!("../../../../docs/architecture/phase-definitions/P2.v1.json");

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(left, _)| *left);
            let mut sorted = serde_json::Map::new();
            for (key, value) in entries {
                sorted.insert(key.clone(), canonical(value));
            }
            Value::Object(sorted)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

fn load(raw: &str) -> Result<PhaseDefinitionDto, AppError> {
    let mut json: Value =
        serde_json::from_str(raw).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
    let expected = json
        .as_object_mut()
        .and_then(|object| object.remove("definitionHash"))
        .and_then(|hash| hash.as_str().map(str::to_owned))
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let bytes = serde_json::to_vec(&canonical(&json))
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    let mut full = json
        .as_object()
        .cloned()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    full.insert("definitionHash".into(), Value::String(expected));
    serde_json::from_value(Value::Object(full))
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}

pub fn get(code: &PhaseCode, version: Option<i64>) -> Result<PhaseDefinitionDto, AppError> {
    let raw = match code {
        PhaseCode::PRE if version.is_none_or(|value| value == 1) => PRE_V1,
        PhaseCode::P1 if version.is_none_or(|value| value == 1) => P1_V1,
        PhaseCode::P2 if version.is_none_or(|value| value == 1) => P2_V1,
        PhaseCode::P3 | PhaseCode::P4 => {
            return Err(AppError::new(ErrorCode::UnsupportedCapability));
        }
        _ => return Err(AppError::new(ErrorCode::NotFound)),
    };
    load(raw)
}

pub fn known_output(code: &PhaseCode, key: &str) -> bool {
    get(code, None).is_ok_and(|definition| {
        definition
            .required_outputs
            .iter()
            .any(|output| output.key == key)
    })
}
