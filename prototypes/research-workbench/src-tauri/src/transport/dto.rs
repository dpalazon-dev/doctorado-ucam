use serde::Deserialize;
fn present_non_null<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}
fn present_nullable<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(deserializer).map(Some)
}
fn required_option<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(deserializer)
}
// Canonical Serde DTOs for CONTRACTS v1. Generated TS is derived ONLY from this Rust module.
use std::collections::BTreeMap;
pub type JsonValue = serde_json::Value;
pub type JsonObject = BTreeMap<String, JsonValue>;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(transparent)]
#[ts(type = "string")]
pub struct UUID(pub String);
impl<'de> serde::Deserialize<'de> for UUID {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = String::deserialize(d)?;
        let parsed = uuid::Uuid::parse_str(&value).map_err(serde::de::Error::custom)?;
        if parsed.to_string() != value {
            return Err(serde::de::Error::custom("UUID must be canonical"));
        }
        Ok(Self(value))
    }
}
impl UUID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}
impl Default for UUID {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(untagged)]
pub enum IpcResult<T> {
    Success(IpcSuccess<T>),
    Failure(IpcFailure),
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IpcSuccess<T> {
    #[ts(type = "1")]
    pub contract_version: ContractVersion,
    pub request_id: UUID,
    #[ts(type = "true")]
    pub ok: SuccessFlag,
    pub data: T,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IpcFailure {
    #[ts(type = "1")]
    pub contract_version: ContractVersion,
    pub request_id: UUID,
    #[ts(type = "false")]
    pub ok: FailureFlag,
    pub error: IpcErrorDto,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, ts_rs::TS)]
#[serde(transparent)]
#[ts(type = "1")]
pub struct ContractVersion(pub u8);
impl<'de> serde::Deserialize<'de> for ContractVersion {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(d)?;
        if v != 1 {
            return Err(serde::de::Error::custom("invalid literal"));
        }
        Ok(Self(v))
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, ts_rs::TS)]
#[serde(transparent)]
#[ts(type = "true")]
pub struct SuccessFlag(pub bool);
impl<'de> serde::Deserialize<'de> for SuccessFlag {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = bool::deserialize(d)?;
        if !v {
            return Err(serde::de::Error::custom("invalid literal"));
        }
        Ok(Self(v))
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, ts_rs::TS)]
#[serde(transparent)]
#[ts(type = "false")]
pub struct FailureFlag(pub bool);
impl<'de> serde::Deserialize<'de> for FailureFlag {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = bool::deserialize(d)?;
        if v {
            return Err(serde::de::Error::custom("invalid literal"));
        }
        Ok(Self(v))
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum IpcErrorCode {
    #[serde(rename = "InvalidInput")]
    InvalidInput,
    #[serde(rename = "NotFound")]
    NotFound,
    #[serde(rename = "Conflict")]
    Conflict,
    #[serde(rename = "DuplicateDecisionRequired")]
    DuplicateDecisionRequired,
    #[serde(rename = "SourceUnreadable")]
    SourceUnreadable,
    #[serde(rename = "InvalidPdf")]
    InvalidPdf,
    #[serde(rename = "StorageUnavailable")]
    StorageUnavailable,
    #[serde(rename = "ImportRecoveryRequired")]
    ImportRecoveryRequired,
    #[serde(rename = "GateBlocked")]
    GateBlocked,
    #[serde(rename = "UnsupportedCapability")]
    UnsupportedCapability,
    #[serde(rename = "PathNotAllowed")]
    PathNotAllowed,
    #[serde(rename = "OperationCancelled")]
    OperationCancelled,
    #[serde(rename = "Busy")]
    Busy,
    #[serde(rename = "IntegrityFailure")]
    IntegrityFailure,
    #[serde(rename = "SchemaTooNew")]
    SchemaTooNew,
    #[serde(rename = "MigrationFailed")]
    MigrationFailed,
    #[serde(rename = "BackupFailed")]
    BackupFailed,
    #[serde(rename = "ExportFailed")]
    ExportFailed,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ReviewType {
    #[serde(rename = "survey")]
    Survey,
    #[serde(rename = "topical_review")]
    TopicalReview,
    #[serde(rename = "slr")]
    Slr,
    #[serde(rename = "mapping_study")]
    MappingStudy,
    #[serde(rename = "tutorial")]
    Tutorial,
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum PaperLifecycle {
    #[serde(rename = "NEW")]
    NEW,
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "COMPLETED")]
    COMPLETED,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
    #[serde(rename = "TRASHED")]
    TRASHED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum PhaseCode {
    #[serde(rename = "PRE")]
    PRE,
    #[serde(rename = "P1")]
    P1,
    #[serde(rename = "P2")]
    P2,
    #[serde(rename = "P3")]
    P3,
    #[serde(rename = "P4")]
    P4,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum PhaseState {
    #[serde(rename = "NOT_STARTED")]
    NOTSTARTED,
    #[serde(rename = "IN_PROGRESS")]
    INPROGRESS,
    #[serde(rename = "COMPLETED")]
    COMPLETED,
    #[serde(rename = "NEEDS_REVIEW")]
    NEEDSREVIEW,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum AnswerResolution {
    #[serde(rename = "PENDING")]
    PENDING,
    #[serde(rename = "ANSWERED")]
    ANSWERED,
    #[serde(rename = "UNKNOWN")]
    UNKNOWN,
    #[serde(rename = "NOT_APPLICABLE")]
    NOTAPPLICABLE,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum Origin {
    #[serde(rename = "literature")]
    Literature,
    #[serde(rename = "researcher_interpretation")]
    ResearcherInterpretation,
    #[serde(rename = "researcher_hypothesis")]
    ResearcherHypothesis,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum Confidence {
    #[serde(rename = "sufficiently_supported")]
    SufficientlySupported,
    #[serde(rename = "context_dependent")]
    ContextDependent,
    #[serde(rename = "uncertain")]
    Uncertain,
    #[serde(rename = "requires_validation")]
    RequiresValidation,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum LocatorState {
    #[serde(rename = "PENDING")]
    PENDING,
    #[serde(rename = "LOCATED")]
    LOCATED,
    #[serde(rename = "STALE")]
    STALE,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum RelevanceRating {
    #[serde(rename = "sufficient")]
    Sufficient,
    #[serde(rename = "use_with_caution")]
    UseWithCaution,
    #[serde(rename = "weak_for_my_purpose")]
    WeakForMyPurpose,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ReadingDecision {
    #[serde(rename = "continue")]
    Continue,
    #[serde(rename = "light_read")]
    LightRead,
    #[serde(rename = "archive")]
    Archive,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelevanceDecisionValue {
    pub relevance: RelevanceRating,
    pub reading_decision: ReadingDecision,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperMetadataInput {
    pub title: String,
    pub authors: Vec<String>,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub year: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub doi: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub venue: Option<String>,
    pub review_type: ReviewType,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum PaperDtoArchivedFromLifecycle {
    #[serde(rename = "NEW")]
    NEW,
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "COMPLETED")]
    COMPLETED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperDto {
    pub title: String,
    pub authors: Vec<String>,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub year: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub doi: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub venue: Option<String>,
    pub review_type: ReviewType,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
    pub id: UUID,
    pub document_id: UUID,
    pub lifecycle: PaperLifecycle,
    #[serde(deserialize_with = "required_option")]
    pub archived_from_lifecycle: Option<PaperDtoArchivedFromLifecycle>,
    #[serde(deserialize_with = "required_option")]
    pub active_phase_code: Option<PhaseCode>,
    pub processing_initialized: bool,
    #[ts(type = "number")]
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(deserialize_with = "required_option")]
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum DocumentDtoStatus {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "SUPERSEDED")]
    SUPERSEDED,
    #[serde(rename = "MISSING")]
    MISSING,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentDto {
    pub id: UUID,
    pub paper_id: UUID,
    pub original_filename: String,
    pub sha256: String,
    pub imported_at: String,
    pub status: DocumentDtoStatus,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum DuplicateCandidateDtoReasonsItem {
    #[serde(rename = "doi")]
    Doi,
    #[serde(rename = "sha256")]
    Sha256,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateCandidateDto {
    pub paper_id: UUID,
    pub title: String,
    pub reasons: Vec<DuplicateCandidateDtoReasonsItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportPreviewDto {
    pub import_token: UUID,
    pub original_filename: String,
    #[ts(type = "number")]
    pub size_bytes: i64,
    pub sha256: String,
    pub candidates: Vec<DuplicateCandidateDto>,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum DuplicateResolutionAction {
    #[serde(rename = "reuseExisting")]
    ReuseExisting,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateResolution {
    pub action: DuplicateResolutionAction,
    pub paper_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum PaperFilterDtoLifecycle {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
    #[serde(rename = "ALL")]
    ALL,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperFilterDto {
    pub lifecycle: PaperFilterDtoLifecycle,
    pub query: String,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub year_from: Option<i64>,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub year_to: Option<i64>,
    pub review_types: Vec<ReviewType>,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub phase: Option<PhaseCode>,
    #[serde(deserialize_with = "required_option")]
    pub cursor: Option<String>,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageDto<T> {
    pub items: Vec<T>,
    #[serde(deserialize_with = "required_option")]
    pub next_cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[serde(deserialize_with = "present_non_null")]
    #[ts(type = "number")]
    pub total: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevisionDto {
    #[ts(type = "number")]
    pub revision: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadingPositionDto {
    pub document_id: UUID,
    #[ts(type = "number")]
    pub page_index: i64,
    pub zoom: f64,
    #[ts(type = "number")]
    pub revision: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenPaperDto {
    pub paper: PaperDto,
    pub document: DocumentDto,
    pub reading_position: ReadingPositionDto,
    pub document_url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseAnswerDto {
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
    pub question_key: String,
    pub answer_text: String,
    #[serde(deserialize_with = "required_option")]
    pub structured_value: Option<JsonValue>,
    pub resolution: AnswerResolution,
    #[serde(deserialize_with = "required_option")]
    pub explanation: Option<String>,
    #[ts(type = "number")]
    pub revision: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum GateIssueDtoStatus {
    #[serde(rename = "missing")]
    Missing,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "invalid")]
    Invalid,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GateIssueDto {
    pub requirement_key: String,
    pub message: String,
    pub status: GateIssueDtoStatus,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GateEvaluationDto {
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
    #[ts(type = "number")]
    pub definition_version: i64,
    pub complete: bool,
    pub issues: Vec<GateIssueDto>,
    #[ts(type = "number")]
    pub phase_revision: i64,
    pub input_snapshot_hash: String,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct P3CandidateDto {
    pub item_id: UUID,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub priority: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct P3CandidateSummaryDto {
    pub paper_id: UUID,
    pub candidates: Vec<P3CandidateDto>,
    #[ts(type = "number")]
    pub candidate_count: i64,
    #[serde(deserialize_with = "required_option")]
    pub no_candidates_justification: Option<String>,
    #[ts(type = "number")]
    pub workflow_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseDto {
    pub paper_id: UUID,
    pub code: PhaseCode,
    #[ts(type = "number")]
    pub definition_version: i64,
    pub state: PhaseState,
    #[ts(type = "number")]
    pub revision: i64,
    #[serde(deserialize_with = "required_option")]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeType {
    #[serde(rename = "concept")]
    Concept,
    #[serde(rename = "claim")]
    Claim,
    #[serde(rename = "evidence")]
    Evidence,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "gap")]
    Gap,
    #[serde(rename = "assumption")]
    Assumption,
    #[serde(rename = "condition")]
    Condition,
    #[serde(rename = "limitation")]
    Limitation,
    #[serde(rename = "method")]
    Method,
    #[serde(rename = "example")]
    Example,
    #[serde(rename = "insight")]
    Insight,
    #[serde(rename = "reference")]
    Reference,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeAttributesQuestionState {
    #[serde(rename = "OPEN")]
    OPEN,
    #[serde(rename = "ANSWERED")]
    ANSWERED,
    #[serde(rename = "DEFERRED")]
    DEFERRED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeAttributesReferenceState {
    #[serde(rename = "TO_REVIEW")]
    TOREVIEW,
    #[serde(rename = "INSPECTED")]
    INSPECTED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "typeCode", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum KnowledgeAttributes {
    #[serde(rename = "claim")]
    Claim {
        #[serde(deserialize_with = "required_option")]
        scope: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations: Option<String>,
    },
    #[serde(rename = "evidence")]
    Evidence {
        evidence_kind: String,
        #[serde(deserialize_with = "required_option")]
        observed_result: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations_text: Option<String>,
    },
    #[serde(rename = "question")]
    Question {
        state: KnowledgeAttributesQuestionState,
        #[serde(deserialize_with = "required_option")]
        answer_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        next_action: Option<String>,
    },
    #[serde(rename = "gap")]
    Gap {
        scope: String,
        justification: String,
        #[serde(deserialize_with = "required_option")]
        search_pending: Option<String>,
    },
    #[serde(rename = "assumption")]
    Assumption {
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "condition")]
    Condition {
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "limitation")]
    Limitation {
        #[serde(deserialize_with = "required_option")]
        effect: Option<String>,
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "method")]
    Method {
        #[serde(deserialize_with = "required_option")]
        family: Option<String>,
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "example")]
    Example {
        #[serde(deserialize_with = "required_option")]
        description: Option<String>,
    },
    #[serde(rename = "insight")]
    Insight {
        baseline: String,
        interpretation: String,
        affected_concept_ids: Vec<UUID>,
    },
    #[serde(rename = "reference")]
    Reference {
        #[serde(deserialize_with = "required_option")]
        identifier: Option<String>,
        reason: String,
        state: KnowledgeAttributesReferenceState,
    },
    #[serde(rename = "concept")]
    Concept {
        preferred_name: String,
        definition: String,
        aliases: Vec<String>,
        #[serde(deserialize_with = "required_option")]
        domain: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemInputTypeCode {
    #[serde(rename = "claim")]
    Claim,
    #[serde(rename = "evidence")]
    Evidence,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "gap")]
    Gap,
    #[serde(rename = "assumption")]
    Assumption,
    #[serde(rename = "condition")]
    Condition,
    #[serde(rename = "limitation")]
    Limitation,
    #[serde(rename = "method")]
    Method,
    #[serde(rename = "example")]
    Example,
    #[serde(rename = "insight")]
    Insight,
    #[serde(rename = "reference")]
    Reference,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemInputAttributesQuestionState {
    #[serde(rename = "OPEN")]
    OPEN,
    #[serde(rename = "ANSWERED")]
    ANSWERED,
    #[serde(rename = "DEFERRED")]
    DEFERRED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemInputAttributesReferenceState {
    #[serde(rename = "TO_REVIEW")]
    TOREVIEW,
    #[serde(rename = "INSPECTED")]
    INSPECTED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "typeCode", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum KnowledgeItemInputAttributes {
    #[serde(rename = "claim")]
    Claim {
        #[serde(deserialize_with = "required_option")]
        scope: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations: Option<String>,
    },
    #[serde(rename = "evidence")]
    Evidence {
        evidence_kind: String,
        #[serde(deserialize_with = "required_option")]
        observed_result: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations_text: Option<String>,
    },
    #[serde(rename = "question")]
    Question {
        state: KnowledgeItemInputAttributesQuestionState,
        #[serde(deserialize_with = "required_option")]
        answer_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        next_action: Option<String>,
    },
    #[serde(rename = "gap")]
    Gap {
        scope: String,
        justification: String,
        #[serde(deserialize_with = "required_option")]
        search_pending: Option<String>,
    },
    #[serde(rename = "assumption")]
    Assumption {
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "condition")]
    Condition {
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "limitation")]
    Limitation {
        #[serde(deserialize_with = "required_option")]
        effect: Option<String>,
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "method")]
    Method {
        #[serde(deserialize_with = "required_option")]
        family: Option<String>,
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "example")]
    Example {
        #[serde(deserialize_with = "required_option")]
        description: Option<String>,
    },
    #[serde(rename = "insight")]
    Insight {
        baseline: String,
        interpretation: String,
        affected_concept_ids: Vec<UUID>,
    },
    #[serde(rename = "reference")]
    Reference {
        #[serde(deserialize_with = "required_option")]
        identifier: Option<String>,
        reason: String,
        state: KnowledgeItemInputAttributesReferenceState,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeItemInput {
    pub type_code: KnowledgeItemInputTypeCode,
    pub title: String,
    pub body_text: String,
    pub origin: Origin,
    pub confidence: Confidence,
    pub attributes: KnowledgeItemInputAttributes,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeAttributesPatchQuestionState {
    #[serde(rename = "OPEN")]
    OPEN,
    #[serde(rename = "ANSWERED")]
    ANSWERED,
    #[serde(rename = "DEFERRED")]
    DEFERRED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeAttributesPatchReferenceState {
    #[serde(rename = "TO_REVIEW")]
    TOREVIEW,
    #[serde(rename = "INSPECTED")]
    INSPECTED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "typeCode", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum KnowledgeAttributesPatch {
    #[serde(rename = "claim")]
    Claim {
        #[serde(deserialize_with = "required_option")]
        scope: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations: Option<String>,
    },
    #[serde(rename = "evidence")]
    Evidence {
        evidence_kind: String,
        #[serde(deserialize_with = "required_option")]
        observed_result: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations_text: Option<String>,
    },
    #[serde(rename = "question")]
    Question {
        state: KnowledgeAttributesPatchQuestionState,
        #[serde(deserialize_with = "required_option")]
        answer_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        next_action: Option<String>,
    },
    #[serde(rename = "gap")]
    Gap {
        scope: String,
        justification: String,
        #[serde(deserialize_with = "required_option")]
        search_pending: Option<String>,
    },
    #[serde(rename = "assumption")]
    Assumption {
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "condition")]
    Condition {
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "limitation")]
    Limitation {
        #[serde(deserialize_with = "required_option")]
        effect: Option<String>,
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "method")]
    Method {
        #[serde(deserialize_with = "required_option")]
        family: Option<String>,
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "example")]
    Example {
        #[serde(deserialize_with = "required_option")]
        description: Option<String>,
    },
    #[serde(rename = "insight")]
    Insight {
        baseline: String,
        interpretation: String,
        affected_concept_ids: Vec<UUID>,
    },
    #[serde(rename = "reference")]
    Reference {
        #[serde(deserialize_with = "required_option")]
        identifier: Option<String>,
        reason: String,
        state: KnowledgeAttributesPatchReferenceState,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemPatchAttributesQuestionState {
    #[serde(rename = "OPEN")]
    OPEN,
    #[serde(rename = "ANSWERED")]
    ANSWERED,
    #[serde(rename = "DEFERRED")]
    DEFERRED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemPatchAttributesReferenceState {
    #[serde(rename = "TO_REVIEW")]
    TOREVIEW,
    #[serde(rename = "INSPECTED")]
    INSPECTED,
    #[serde(rename = "DISMISSED")]
    DISMISSED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "typeCode", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum KnowledgeItemPatchAttributes {
    #[serde(rename = "claim")]
    Claim {
        #[serde(deserialize_with = "required_option")]
        scope: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations: Option<String>,
    },
    #[serde(rename = "evidence")]
    Evidence {
        evidence_kind: String,
        #[serde(deserialize_with = "required_option")]
        observed_result: Option<String>,
        #[serde(deserialize_with = "required_option")]
        conditions_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        limitations_text: Option<String>,
    },
    #[serde(rename = "question")]
    Question {
        state: KnowledgeItemPatchAttributesQuestionState,
        #[serde(deserialize_with = "required_option")]
        answer_text: Option<String>,
        #[serde(deserialize_with = "required_option")]
        next_action: Option<String>,
    },
    #[serde(rename = "gap")]
    Gap {
        scope: String,
        justification: String,
        #[serde(deserialize_with = "required_option")]
        search_pending: Option<String>,
    },
    #[serde(rename = "assumption")]
    Assumption {
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "condition")]
    Condition {
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "limitation")]
    Limitation {
        #[serde(deserialize_with = "required_option")]
        effect: Option<String>,
        #[serde(deserialize_with = "required_option")]
        applicability_scope: Option<String>,
    },
    #[serde(rename = "method")]
    Method {
        #[serde(deserialize_with = "required_option")]
        family: Option<String>,
        #[serde(deserialize_with = "required_option")]
        context: Option<String>,
    },
    #[serde(rename = "example")]
    Example {
        #[serde(deserialize_with = "required_option")]
        description: Option<String>,
    },
    #[serde(rename = "insight")]
    Insight {
        baseline: String,
        interpretation: String,
        affected_concept_ids: Vec<UUID>,
    },
    #[serde(rename = "reference")]
    Reference {
        #[serde(deserialize_with = "required_option")]
        identifier: Option<String>,
        reason: String,
        state: KnowledgeItemPatchAttributesReferenceState,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeItemPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub body_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub confidence: Option<Confidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub attributes: Option<KnowledgeItemPatchAttributes>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeFilterLifecycle {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
    #[serde(rename = "ALL")]
    ALL,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeFilter {
    #[serde(deserialize_with = "required_option")]
    pub paper_id: Option<UUID>,
    #[serde(deserialize_with = "required_option")]
    pub concept_id: Option<UUID>,
    pub type_codes: Vec<KnowledgeType>,
    pub lifecycle: KnowledgeFilterLifecycle,
    pub include_archived: bool,
    #[serde(deserialize_with = "required_option")]
    pub confidence: Option<Confidence>,
    #[serde(deserialize_with = "required_option")]
    pub origin: Option<Origin>,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceSummaryDto {
    pub id: UUID,
    pub document_id: UUID,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub page_index: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub page_label: Option<String>,
    pub locator_state: LocatorState,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemDtoBodyFormat {
    #[serde(rename = "plain_text")]
    PlainText,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemDtoLifecycle {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum KnowledgeItemDtoProvenanceStatus {
    #[serde(rename = "NONE")]
    NONE,
    #[serde(rename = "PENDING")]
    PENDING,
    #[serde(rename = "LOCATED")]
    LOCATED,
    #[serde(rename = "STALE")]
    STALE,
    #[serde(rename = "MIXED")]
    MIXED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeItemDto {
    pub id: UUID,
    pub type_code: KnowledgeType,
    pub title: String,
    pub body_text: String,
    pub body_format: KnowledgeItemDtoBodyFormat,
    pub body_json: (),
    pub origin: Origin,
    pub lifecycle: KnowledgeItemDtoLifecycle,
    pub confidence: Confidence,
    pub attributes: KnowledgeAttributes,
    pub paper_ids: Vec<UUID>,
    pub concept_ids: Vec<UUID>,
    pub provenance: Vec<ProvenanceSummaryDto>,
    pub provenance_status: KnowledgeItemDtoProvenanceStatus,
    #[ts(type = "number")]
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ConceptDtoBodyFormat {
    #[serde(rename = "plain_text")]
    PlainText,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ConceptDtoLifecycle {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ConceptDtoProvenanceStatus {
    #[serde(rename = "NONE")]
    NONE,
    #[serde(rename = "PENDING")]
    PENDING,
    #[serde(rename = "LOCATED")]
    LOCATED,
    #[serde(rename = "STALE")]
    STALE,
    #[serde(rename = "MIXED")]
    MIXED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ConceptDtoTypeCode {
    #[serde(rename = "concept")]
    Concept,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ConceptDtoAttributesTypeCode {
    #[serde(rename = "concept")]
    Concept,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptDtoAttributes {
    pub type_code: ConceptDtoAttributesTypeCode,
    pub preferred_name: String,
    pub definition: String,
    pub aliases: Vec<String>,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptDto {
    pub id: UUID,
    pub title: String,
    pub body_text: String,
    pub body_format: ConceptDtoBodyFormat,
    pub body_json: (),
    pub origin: Origin,
    pub lifecycle: ConceptDtoLifecycle,
    pub confidence: Confidence,
    pub paper_ids: Vec<UUID>,
    pub concept_ids: Vec<UUID>,
    pub provenance: Vec<ProvenanceSummaryDto>,
    pub provenance_status: ConceptDtoProvenanceStatus,
    #[ts(type = "number")]
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
    pub type_code: ConceptDtoTypeCode,
    pub attributes: ConceptDtoAttributes,
    pub preferred_name: String,
    pub normalized_name: String,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
    pub aliases: Vec<String>,
    pub merged_into_id: (),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceDto {
    pub id: UUID,
    pub document_id: UUID,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub page_index: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub page_label: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub section: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub quote_text: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub locator: Option<JsonObject>,
    pub captured_document_hash: String,
    pub locator_state: LocatorState,
    #[ts(type = "number")]
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum RelationType {
    #[serde(rename = "supports")]
    Supports,
    #[serde(rename = "contradicts")]
    Contradicts,
    #[serde(rename = "extends")]
    Extends,
    #[serde(rename = "causes")]
    Causes,
    #[serde(rename = "requires")]
    Requires,
    #[serde(rename = "depends_on")]
    DependsOn,
    #[serde(rename = "works_when")]
    WorksWhen,
    #[serde(rename = "fails_when")]
    FailsWhen,
    #[serde(rename = "compares_with")]
    ComparesWith,
    #[serde(rename = "part_of")]
    PartOf,
    #[serde(rename = "similar_to")]
    SimilarTo,
    #[serde(rename = "limits")]
    Limits,
    #[serde(rename = "improves")]
    Improves,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum RelationDtoLifecycle {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationDto {
    pub id: UUID,
    pub source_item_id: UUID,
    pub target_item_id: UUID,
    pub type_code: RelationType,
    pub context_text: String,
    pub justification_text: String,
    pub origin: Origin,
    pub confidence: Confidence,
    pub lifecycle: RelationDtoLifecycle,
    pub provenance_ids: Vec<UUID>,
    #[ts(type = "number")]
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceInput {
    pub document_id: UUID,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub page_index: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub page_label: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub section: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub quote_text: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub locator: Option<JsonObject>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum SearchRequestDtoScopesItem {
    #[serde(rename = "papers")]
    Papers,
    #[serde(rename = "knowledge")]
    Knowledge,
    #[serde(rename = "concepts")]
    Concepts,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum SearchRequestDtoFiltersLifecycle {
    #[serde(rename = "ACTIVE")]
    ACTIVE,
    #[serde(rename = "ARCHIVED")]
    ARCHIVED,
    #[serde(rename = "ALL")]
    ALL,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchRequestDtoFilters {
    #[serde(deserialize_with = "required_option")]
    pub paper_id: Option<UUID>,
    #[serde(deserialize_with = "required_option")]
    pub concept_id: Option<UUID>,
    pub type_codes: Vec<KnowledgeType>,
    pub lifecycle: SearchRequestDtoFiltersLifecycle,
    pub include_archived: bool,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub confidence: Option<Confidence>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchRequestDto {
    pub query: String,
    pub scopes: Vec<SearchRequestDtoScopesItem>,
    pub filters: SearchRequestDtoFilters,
    #[serde(deserialize_with = "required_option")]
    pub cursor: Option<String>,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum SearchHitDtoEntityType {
    #[serde(rename = "paper")]
    Paper,
    #[serde(rename = "knowledgeItem")]
    KnowledgeItem,
    #[serde(rename = "concept")]
    Concept,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchHitDto {
    pub entity_type: SearchHitDtoEntityType,
    pub entity_id: UUID,
    pub title: String,
    pub excerpt: String,
    #[serde(deserialize_with = "required_option")]
    pub paper_id: Option<UUID>,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub page_index: Option<i64>,
    pub rank: f64,
    pub lifecycle: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum ExportRequestDtoFormat {
    #[serde(rename = "jsonl_markdown")]
    JsonlMarkdown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportRequestDto {
    pub destination_token: UUID,
    pub include_pdfs: bool,
    pub format: ExportRequestDtoFormat,
    #[serde(deserialize_with = "required_option")]
    pub paper_ids: Option<Vec<UUID>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportResultDto {
    pub export_id: UUID,
    pub output_name: String,
    pub manifest_sha256: String,
    #[ts(type = "number")]
    pub file_count: i64,
    pub entity_counts: std::collections::BTreeMap<String, i64>,
    pub included_pdfs: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupDto {
    pub backup_id: UUID,
    pub created_at: String,
    #[ts(type = "number")]
    pub schema_version: i64,
    pub verified: bool,
    pub manifest_sha256: String,
    #[ts(type = "number")]
    pub size_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum LongOperationState {
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum LongOperationDtoResultKind {
    #[serde(rename = "export")]
    Export,
    #[serde(rename = "backup")]
    Backup,
    #[serde(rename = "restore")]
    Restore,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(untagged)]
pub enum LongOperationDtoResultValue {
    V0(ExportResultDto),
    V1(BackupDto),
    V2(PreparedLibraryDto),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LongOperationDtoResult {
    pub kind: LongOperationDtoResultKind,
    pub value: LongOperationDtoResultValue,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LongOperationDtoError {
    pub code: IpcErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LongOperationDto {
    pub operation_id: UUID,
    pub state: LongOperationState,
    #[serde(deserialize_with = "required_option")]
    pub progress: Option<f64>,
    #[serde(deserialize_with = "required_option")]
    pub result: Option<LongOperationDtoResult>,
    #[serde(deserialize_with = "required_option")]
    pub error: Option<LongOperationDtoError>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportDestinationDto {
    pub destination_token: UUID,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupDestinationDto {
    pub destination_token: UUID,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestoreTargetDto {
    pub target_token: UUID,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparedLibraryDto {
    pub prepared_library_token: UUID,
    pub library_id: UUID,
    pub display_name: String,
    #[ts(type = "number")]
    pub schema_version: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum LibrarySwitchTarget {
    #[serde(rename = "selected")]
    Selected { target_token: UUID },
    #[serde(rename = "restored")]
    Restored { prepared_library_token: UUID },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum AppInfoDtoState {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "readOnlyDiagnostic")]
    ReadOnlyDiagnostic,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppInfoDto {
    pub app_version: String,
    pub contract_version: ContractVersion,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub schema_version: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub library_id: Option<UUID>,
    #[serde(deserialize_with = "required_option")]
    pub library_root_label: Option<String>,
    pub state: AppInfoDtoState,
    pub capabilities: std::collections::BTreeMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum PhaseRuleKey {
    #[serde(rename = "answerProcessed")]
    AnswerProcessed,
    #[serde(rename = "paperHasActiveDocument")]
    PaperHasActiveDocument,
    #[serde(rename = "p1DecisionProcessed")]
    P1DecisionProcessed,
    #[serde(rename = "p2ArtifactPresent")]
    P2ArtifactPresent,
    #[serde(rename = "p2NoCandidatesJustified")]
    P2NoCandidatesJustified,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseRequiredOutputDto {
    pub key: String,
    pub label: String,
    pub prompt: String,
    pub required: bool,
    pub allowed_resolutions: Vec<AnswerResolution>,
    pub rule_key: PhaseRuleKey,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseDefinitionDto {
    pub code: PhaseCode,
    #[ts(type = "number")]
    pub version: i64,
    pub name: String,
    pub objective: String,
    pub key_questions: Vec<String>,
    pub do_items: Vec<String>,
    pub dont_items: Vec<String>,
    pub considerations: Vec<String>,
    pub required_outputs: Vec<PhaseRequiredOutputDto>,
    pub completion_rules: Vec<PhaseRuleKey>,
    pub definition_hash: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryInfoDto {
    pub library_id: UUID,
    pub display_name: String,
    pub root_label: String,
    #[ts(type = "number")]
    pub schema_version: i64,
    pub writable: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibrarySelectPdfArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryConfirmImportArgs {
    pub request_id: UUID,
    pub import_token: UUID,
    pub metadata: PaperMetadataInput,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duplicate_resolution: Option<DuplicateResolution>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryCancelImportArgs {
    pub request_id: UUID,
    pub import_token: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryListPapersArgs {
    pub request_id: UUID,
    pub filter: PaperFilterDto,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryGetPaperArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryUpdateMetadataArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub metadata: PaperMetadataInput,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryArchivePaperArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryRestorePaperArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReaderOpenPaperArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReaderGetLastOpenedPaperArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReaderGetReadingPositionArgs {
    pub request_id: UUID,
    pub document_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReaderSaveReadingPositionArgs {
    pub request_id: UUID,
    pub document_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
    #[ts(type = "number")]
    pub page_index: i64,
    pub zoom: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowGetPhaseArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowGetPhaseAnswersArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowGetPhaseDefinitionArgs {
    pub request_id: UUID,
    pub phase_code: PhaseCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[serde(deserialize_with = "present_non_null")]
    #[ts(type = "number")]
    pub version: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowSavePhaseAnswerArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
    pub question_key: String,
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub answer_text: String,
    #[serde(deserialize_with = "required_option")]
    pub structured_value: Option<JsonValue>,
    pub resolution: AnswerResolution,
    #[serde(deserialize_with = "required_option")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowEvaluateGateArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowAdvancePhaseArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub from_phase: PhaseCode,
    #[ts(type = "number")]
    pub expected_phase_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowAdvancePhaseOutput {
    pub phase: PhaseDto,
    pub gate: GateEvaluationDto,
    #[serde(deserialize_with = "required_option")]
    pub next_phase: Option<PhaseCode>,
    pub paper_lifecycle: PaperLifecycle,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowGoBackToPhaseArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub target_phase: PhaseCode,
    #[ts(type = "number")]
    pub expected_active_phase_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowGoBackToPhaseOutput {
    pub active_phase_code: PhaseCode,
    #[ts(type = "number")]
    pub active_phase_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowTouchPhaseArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub phase_code: PhaseCode,
    #[ts(type = "number")]
    pub expected_active_phase_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowSetP3CandidateArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
    pub item_id: UUID,
    pub selected: bool,
    #[ts(type = "number | null")]
    #[serde(deserialize_with = "required_option")]
    pub priority: Option<i64>,
    #[serde(deserialize_with = "required_option")]
    pub rationale: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub no_candidates_justification: Option<String>,
    #[ts(type = "number")]
    pub expected_workflow_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowGetP3CandidateSummaryArgs {
    pub request_id: UUID,
    pub paper_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeCreateItemArgs {
    pub request_id: UUID,
    pub item: KnowledgeItemInput,
    pub paper_ids: Vec<UUID>,
    pub concept_ids: Vec<UUID>,
    pub provenance: Vec<ProvenanceInput>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeUpdateItemArgs {
    pub request_id: UUID,
    pub item_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub patch: KnowledgeItemPatch,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeArchiveItemArgs {
    pub request_id: UUID,
    pub item_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeRestoreItemArgs {
    pub request_id: UUID,
    pub item_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeGetItemArgs {
    pub request_id: UUID,
    pub item_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeListItemsArgs {
    pub request_id: UUID,
    pub filter: KnowledgeFilter,
    #[serde(deserialize_with = "required_option")]
    pub cursor: Option<String>,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptSuggestArgs {
    pub request_id: UUID,
    pub query: String,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptGetArgs {
    pub request_id: UUID,
    pub concept_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptListItemsArgs {
    pub request_id: UUID,
    pub concept_id: UUID,
    pub filter: KnowledgeFilter,
    #[serde(deserialize_with = "required_option")]
    pub cursor: Option<String>,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptCreateArgs {
    pub request_id: UUID,
    pub preferred_name: String,
    pub definition: String,
    pub aliases: Vec<String>,
    #[serde(deserialize_with = "required_option")]
    pub domain: Option<String>,
    pub origin: Origin,
    pub confidence: Confidence,
    pub provenance: Vec<ProvenanceInput>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptUpdateArgs {
    pub request_id: UUID,
    pub concept_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferred_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub aliases: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[serde(deserialize_with = "present_nullable")]
    pub domain: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptLinkArgs {
    pub request_id: UUID,
    pub item_id: UUID,
    pub concept_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptArchiveArgs {
    pub request_id: UUID,
    pub concept_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConceptRestoreArgs {
    pub request_id: UUID,
    pub concept_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationCreateArgs {
    pub request_id: UUID,
    pub source_item_id: UUID,
    pub target_item_id: UUID,
    pub type_code: RelationType,
    pub context_text: String,
    pub origin: Origin,
    pub justification_text: String,
    pub confidence: Confidence,
    pub provenance_ids: Vec<UUID>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationUpdateArgs {
    pub request_id: UUID,
    pub relation_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub context_text: String,
    pub justification_text: String,
    pub origin: Origin,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationListArgs {
    pub request_id: UUID,
    pub item_id: UUID,
    pub include_archived: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationArchiveArgs {
    pub request_id: UUID,
    pub relation_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationRestoreArgs {
    pub request_id: UUID,
    pub relation_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceAttachLocatorArgs {
    pub request_id: UUID,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub item_id: Option<UUID>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub relation_id: Option<UUID>,
    #[ts(type = "number")]
    pub expected_parent_revision: i64,
    pub input: ProvenanceInput,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceUpdateLocatorArgs {
    pub request_id: UUID,
    pub provenance_id: UUID,
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub input: ProvenanceInput,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceGetArgs {
    pub request_id: UUID,
    pub provenance_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceCheckDocumentHashArgs {
    pub request_id: UUID,
    pub document_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceCheckDocumentHashOutput {
    pub current_hash: String,
    pub stale_provenance_ids: Vec<UUID>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchLibraryArgs {
    pub request_id: UUID,
    pub request: SearchRequestDto,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchKnowledgeArgs {
    pub request_id: UUID,
    pub request: SearchRequestDto,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportChooseDestinationArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportLibraryArgs {
    pub request_id: UUID,
    pub operation_id: UUID,
    pub request: ExportRequestDto,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportPaperArgs {
    pub request_id: UUID,
    pub operation_id: UUID,
    pub paper_id: UUID,
    pub destination_token: UUID,
    pub include_pdf: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupChooseDestinationArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupCreateArgs {
    pub request_id: UUID,
    pub operation_id: UUID,
    pub destination_token: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupSelectArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupSelectOutput {
    pub backup_token: UUID,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupChooseRestoreTargetArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationGetStatusArgs {
    pub request_id: UUID,
    pub operation_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationCancelArgs {
    pub request_id: UUID,
    pub operation_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationCancelOutput {
    pub cancellation_requested: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupVerifyArgs {
    pub request_id: UUID,
    pub backup_token: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupVerifyOutput {
    pub valid: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupRestoreArgs {
    pub request_id: UUID,
    pub operation_id: UUID,
    pub backup_token: UUID,
    pub target_token: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsGetAppInfoArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsGetLibraryInfoArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsSelectLibraryArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsSelectLibraryOutput {
    pub target_token: UUID,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsSwitchLibraryArgs {
    pub request_id: UUID,
    pub target: LibrarySwitchTarget,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsGetLibraryStatusArgs {
    pub request_id: UUID,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsGetLibraryStatusOutput {
    pub writable: bool,
    #[ts(type = "number")]
    pub active_operations: i64,
    pub recovery_required: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IpcErrorDto {
    pub code: IpcErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<JsonObject>,
}
