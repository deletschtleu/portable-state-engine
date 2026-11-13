//! Structured error categories for the Portable State Engine (`FR-004`, `FR-013`, `NFR-006`).

use std::fmt;

/// High-level error classification required by `FR-013`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    ValidationRejection,
    InvalidAggregateId,
    AggregateNotFound,
    RevisionConflict,
    InvalidRevision,
    CorruptedStream,
    StorageFailure,
}

/// Domain-level rejection returned by `decide` when a command violates business rules (`FR-004`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    pub code: String,
    pub message: String,
}

impl DomainError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

/// Engine-wide error model distinguishing domain rejections, invalid IDs/not-found,
/// concurrency conflicts, invalid historical revisions, stream corruption, and storage failures (`FR-013`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    DomainRejection {
        aggregate_id: Option<String>,
        error: DomainError,
    },
    InvalidAggregateId {
        aggregate_id: String,
        reason: String,
    },
    AggregateNotFound {
        aggregate_id: String,
    },
    RevisionConflict {
        aggregate_id: String,
        expected_revision: u64,
        actual_revision: u64,
    },
    InvalidRevision {
        aggregate_id: Option<String>,
        requested_revision: i64,
        current_revision: u64,
        reason: String,
    },
    CorruptedStream {
        aggregate_id: Option<String>,
        expected_revision: u64,
        actual_revision: u64,
        reason: String,
    },
    StorageFailure {
        aggregate_id: Option<String>,
        operation: String,
        reason: String,
    },
}

impl EngineError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::DomainRejection { .. } => ErrorCategory::ValidationRejection,
            Self::InvalidAggregateId { .. } => ErrorCategory::InvalidAggregateId,
            Self::AggregateNotFound { .. } => ErrorCategory::AggregateNotFound,
            Self::RevisionConflict { .. } => ErrorCategory::RevisionConflict,
            Self::InvalidRevision { .. } => ErrorCategory::InvalidRevision,
            Self::CorruptedStream { .. } => ErrorCategory::CorruptedStream,
            Self::StorageFailure { .. } => ErrorCategory::StorageFailure,
        }
    }

    pub fn domain_rejection(aggregate_id: Option<impl Into<String>>, error: DomainError) -> Self {
        Self::DomainRejection {
            aggregate_id: aggregate_id.map(Into::into),
            error,
        }
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DomainRejection {
                aggregate_id: Some(agg),
                error,
            } => write!(f, "domain rejection for aggregate '{}': {}", agg, error),
            Self::DomainRejection {
                aggregate_id: None,
                error,
            } => write!(f, "domain rejection: {}", error),
            Self::InvalidAggregateId {
                aggregate_id,
                reason,
            } => write!(f, "invalid aggregate_id '{}': {}", aggregate_id, reason),
            Self::AggregateNotFound { aggregate_id } => {
                write!(f, "aggregate '{}' not found", aggregate_id)
            }
            Self::RevisionConflict {
                aggregate_id,
                expected_revision,
                actual_revision,
            } => write!(
                f,
                "revision conflict on aggregate '{}': expected {}, actual {}",
                aggregate_id, expected_revision, actual_revision
            ),
            Self::InvalidRevision {
                aggregate_id,
                requested_revision,
                current_revision,
                reason,
            } => write!(
                f,
                "invalid revision {} (current {}) for {:?}: {}",
                requested_revision, current_revision, aggregate_id, reason
            ),
            Self::CorruptedStream {
                aggregate_id,
                expected_revision,
                actual_revision,
                reason,
            } => write!(
                f,
                "corrupted stream for {:?}: expected revision {}, actual {} ({})",
                aggregate_id, expected_revision, actual_revision, reason
            ),
            Self::StorageFailure {
                aggregate_id,
                operation,
                reason,
            } => write!(
                f,
                "storage failure during '{}' on {:?}: {}",
                operation, aggregate_id, reason
            ),
        }
    }
}

impl std::error::Error for EngineError {}
