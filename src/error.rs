//! Error types for the ergonomic layer.
//!
//! Kept std-only (no `thiserror`) so the ergonomic layer does not introduce
//! new dependencies. Variants are coarse-grained enough that each one
//! captures a single VDA 5050 spec rule.

use std::fmt;

/// Validation failures returned by `validate()` methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A header field failed validation (missing `manufacturer` etc.).
    Header(HeaderError),
    /// `Header.timestamp` could not be parsed or formatted as ISO 8601 UTC.
    Timestamp(TimestampError),
    /// A `Zone` is missing a field that its `ZoneType` requires.
    ZoneMissingField { zone_id: String, field: &'static str },
    /// A polygon has fewer than three vertices.
    PolygonTooSmall { context: &'static str, len: usize },
    /// A `Trajectory` knot-vector length is wrong for its control points.
    KnotVectorMismatch {
        got: usize,
        expected: usize,
        control_points: usize,
        degree: u32,
    },
    /// An `InstantAction` was given a non-`BlockingNone` blocking type.
    InstantActionBlocking { action_id: String },
    /// An `Action` failed validation (e.g. missing `action_id`).
    Action(ActionError),
    /// An `Order` has no nodes and no edges.
    EmptyOrder,
    /// A `BoundingBoxReference` had `z` set as required; reserved for future use.
    /// `PowerSupply` had an out-of-range value.
    OutOfRange { field: &'static str, value: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header(e) => write!(f, "header: {e}"),
            Self::Timestamp(e) => write!(f, "timestamp: {e}"),
            Self::ZoneMissingField { zone_id, field } => {
                write!(f, "zone {zone_id:?} is missing required field {field}")
            }
            Self::PolygonTooSmall { context, len } => {
                write!(f, "{context} has only {len} vertices; minimum 3 required")
            }
            Self::KnotVectorMismatch { got, expected, control_points, degree } => write!(
                f,
                "knot vector length {got} != control_points ({control_points}) + degree ({degree}) + 1 ({expected})"
            ),
            Self::InstantActionBlocking { action_id } => {
                write!(f, "instant action {action_id:?} must have blocking_type = NONE")
            }
            Self::Action(e) => write!(f, "action: {e}"),
            Self::EmptyOrder => write!(f, "order has no nodes and no edges"),
            Self::OutOfRange { field, value } => {
                write!(f, "{field}={value} is out of range")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<HeaderError> for ValidationError {
    fn from(e: HeaderError) -> Self {
        Self::Header(e)
    }
}

impl From<TimestampError> for ValidationError {
    fn from(e: TimestampError) -> Self {
        Self::Timestamp(e)
    }
}

impl From<ActionError> for ValidationError {
    fn from(e: ActionError) -> Self {
        Self::Action(e)
    }
}

/// `Header` validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    /// `manufacturer` is empty.
    MissingManufacturer,
    /// `serial_number` is empty.
    MissingSerialNumber,
    /// `version` is empty.
    MissingVersion,
    /// `version` does not start with `"3."`.
    UnsupportedVersion(String),
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingManufacturer => write!(f, "manufacturer is empty"),
            Self::MissingSerialNumber => write!(f, "serial_number is empty"),
            Self::MissingVersion => write!(f, "version is empty"),
            Self::UnsupportedVersion(v) => write!(f, "version {v:?} is not a v3.x release"),
        }
    }
}

impl std::error::Error for HeaderError {}

/// `Timestamp` (ISO 8601) parse / format failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimestampError {
    /// The input does not match `YYYY-MM-DDTHH:MM:SS[.fff]Z`.
    BadFormat(String),
    /// One of the date / time components did not parse.
    BadComponent(String),
}

impl fmt::Display for TimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFormat(s) => write!(f, "timestamp {s:?} is not ISO 8601 UTC"),
            Self::BadComponent(s) => write!(f, "timestamp component {s:?} failed to parse"),
        }
    }
}

impl std::error::Error for TimestampError {}

/// `Action` validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    /// `action_type` is empty.
    MissingActionType,
    /// `action_id` is empty.
    MissingActionId,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingActionType => write!(f, "action_type is empty"),
            Self::MissingActionId => write!(f, "action_id is empty"),
        }
    }
}

impl std::error::Error for ActionError {}

/// Returned when a proto enum tag has no known variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownVariant {
    pub enum_name: &'static str,
    pub value: i32,
}

impl fmt::Display for UnknownVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown {} variant: {}", self.enum_name, self.value)
    }
}

impl std::error::Error for UnknownVariant {}
