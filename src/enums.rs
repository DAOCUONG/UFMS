//! Ergonomic helpers for every proto enum under `vda5050.v3`.
//!
//! For each generated enum we add a small set of additional methods next to
//! the prost-generated [`as_str_name`](prost::Enumeration::as_str_name) and
//! [`from_str_name`](prost::Enumeration::from_str_name):
//!
//! - [`short_name`](EnumExt::short_name) — lowercase, no enum-prefix
//!   ("online" instead of "CONNECTION_ONLINE"), suitable for log output.
//! - [`is_unspecified`](EnumExt::is_unspecified) — true for the `0` variant.
//! - [`Display`] — uses `short_name`.
//! - [`FromStr`] — accepts both `as_str_name()` and `short_name()` forms.

use std::fmt;
use std::str::FromStr;

use crate::error::UnknownVariant;
use crate::vda5050::v3::{
    ActionScope, ActionStatus, ActionType, BidirectedLimitation, BlockingType, ConnectionState,
    CorridorReferencePoint, DirectedLimitation, ErrorLevel, EStopType, GrantType, InfoLevel,
    MapStatus, OperatingMode, OptionalParameterSupport, OrientationType, ReleaseLossBehavior,
    RequestStatus, RequestType, ValueDataType, ZoneType,
};

/// Common helpers for every proto-generated enum.
pub trait EnumExt: Copy + Eq + fmt::Debug + FromStr<Err = UnknownVariant> {
    /// Lowercase, no enum-prefix short name (e.g., `"online"` instead of
    /// `"CONNECTION_ONLINE"`). The unspecified variant reads
    /// `"unspecified"`.
    fn short_name(&self) -> &'static str;

    /// True if this is the `_UNSPECIFIED = 0` sentinel variant.
    fn is_unspecified(&self) -> bool {
        self.short_name() == "unspecified"
    }
}

// ---------------------------------------------------------------------------
// Per-enum FromStr / Display / EnumExt impls.
// ---------------------------------------------------------------------------
//
// All variants of all 16 enums are listed verbatim. The `short_name` table is
// the only piece of bespoke logic; the `Display` + `FromStr` impls are
// mechanical.

macro_rules! impl_enum_str {
    ($enum:ty, $tag:literal, $($variant:ident => $short:literal),+ $(,)?) => {
        impl EnumExt for $enum {
            fn short_name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $short),+
                }
            }
        }

        impl fmt::Display for $enum {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.short_name())
            }
        }

        impl FromStr for $enum {
            type Err = UnknownVariant;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                // Accept three forms, in priority order:
                // 1. Canonical protobuf form: "BLOCKING_HARD"
                // 2. Case-insensitive canonical: "blocking_hard"
                // 3. Short lowercase name from `short_name()`: "hard"
                if let Some(v) = Self::from_str_name(s) {
                    return Ok(v);
                }
                let up = s.to_ascii_uppercase();
                if let Some(v) = Self::from_str_name(&up) {
                    return Ok(v);
                }
                match s {
                    $($short => Ok(Self::$variant),)+
                    _ => Err(UnknownVariant { enum_name: $tag, value: -1 }),
                }
            }
        }
    };
}

impl_enum_str!(
    ConnectionState,
    "ConnectionState",
    Unspecified => "unspecified",
    ConnectionOnline => "online",
    ConnectionOffline => "offline",
    ConnectionHibernating => "hibernating",
    ConnectionBroken => "broken",
);

impl_enum_str!(
    BlockingType,
    "BlockingType",
    Unspecified => "unspecified",
    BlockingNone => "none",
    BlockingSoft => "soft",
    BlockingSingle => "single",
    BlockingHard => "hard",
);

impl_enum_str!(
    ActionStatus,
    "ActionStatus",
    Unspecified => "unspecified",
    ActionWaiting => "waiting",
    ActionInitializing => "initializing",
    ActionRunning => "running",
    ActionPaused => "paused",
    ActionRetriable => "retriable",
    ActionFinished => "finished",
    ActionFailed => "failed",
);

impl_enum_str!(
    OperatingMode,
    "OperatingMode",
    Unspecified => "unspecified",
    OperatingStartup => "startup",
    OperatingAutomatic => "automatic",
    OperatingSemiautomatic => "semiautomatic",
    OperatingIntervened => "intervened",
    OperatingManual => "manual",
    OperatingService => "service",
    OperatingTeachIn => "teach_in",
);

impl_enum_str!(
    ErrorLevel,
    "ErrorLevel",
    Unspecified => "unspecified",
    ErrorWarning => "warning",
    ErrorUrgent => "urgent",
    ErrorCritical => "critical",
    ErrorFatal => "fatal",
);

impl_enum_str!(
    InfoLevel,
    "InfoLevel",
    Unspecified => "unspecified",
    InfoInfo => "info",
    InfoDebug => "debug",
);

impl_enum_str!(
    EStopType,
    "EStopType",
    EstopTypeUnspecified => "unspecified",
    EstopManual => "manual",
    EstopRemote => "remote",
    EstopNone => "none",
);

impl_enum_str!(
    MapStatus,
    "MapStatus",
    Unspecified => "unspecified",
    MapEnabled => "enabled",
    MapDisabled => "disabled",
);

impl_enum_str!(
    OrientationType,
    "OrientationType",
    Unspecified => "unspecified",
    OrientationGlobal => "global",
    OrientationTangential => "tangential",
);

impl_enum_str!(
    CorridorReferencePoint,
    "CorridorReferencePoint",
    Unspecified => "unspecified",
    CorridorKinematicCenter => "kinematic_center",
    CorridorContour => "contour",
);

impl_enum_str!(
    ReleaseLossBehavior,
    "ReleaseLossBehavior",
    Unspecified => "unspecified",
    ReleaseStop => "stop",
    ReleaseReturn => "return",
    ReleaseContinue => "continue",
    ReleaseEvacuate => "evacuate",
);

impl_enum_str!(
    ZoneType,
    "ZoneType",
    Unspecified => "unspecified",
    ZoneBlocked => "blocked",
    ZoneLineGuided => "line_guided",
    ZoneRelease => "release",
    ZoneCoordinatedReplanning => "coordinated_replanning",
    ZoneSpeedLimit => "speed_limit",
    ZoneAction => "action",
    ZonePriority => "priority",
    ZonePenalty => "penalty",
    ZoneDirected => "directed",
    ZoneBidirected => "bidirected",
);

impl_enum_str!(
    DirectedLimitation,
    "DirectedLimitation",
    Unspecified => "unspecified",
    DirectedSoft => "soft",
    DirectedRestricted => "restricted",
    DirectedStrict => "strict",
);

impl_enum_str!(
    BidirectedLimitation,
    "BidirectedLimitation",
    Unspecified => "unspecified",
    BidirectedSoft => "soft",
    BidirectedRestricted => "restricted",
);

impl_enum_str!(
    RequestType,
    "RequestType",
    Unspecified => "unspecified",
    RequestAccess => "access",
    RequestReplanning => "replanning",
    RequestCorridor => "corridor",
);

impl_enum_str!(
    RequestStatus,
    "RequestStatus",
    Unspecified => "unspecified",
    RequestRequested => "requested",
    RequestGranted => "granted",
    RequestRevoked => "revoked",
    RequestExpired => "expired",
);

impl_enum_str!(
    GrantType,
    "GrantType",
    Unspecified => "unspecified",
    GrantGranted => "granted",
    GrantQueued => "queued",
    GrantRevoked => "revoked",
    GrantRejected => "rejected",
);

impl_enum_str!(
    ActionScope,
    "ActionScope",
    Unspecified => "unspecified",
    ScopeInstant => "instant",
    ScopeNode => "node",
    ScopeEdge => "edge",
    ScopeZone => "zone",
);

impl_enum_str!(
    ValueDataType,
    "ValueDataType",
    Unspecified => "unspecified",
    ValueBool => "bool",
    ValueNumber => "number",
    ValueInteger => "integer",
    ValueString => "string",
    ValueObject => "object",
    ValueArray => "array",
);

impl_enum_str!(
    OptionalParameterSupport,
    "OptionalParameterSupport",
    Unspecified => "unspecified",
    OptionalSupported => "supported",
    OptionalRequired => "required",
);

// `ActionType` is reserved for future typed extension; only the unspecified
// variant exists today, but we still wire up the same surface for uniformity.
impl_enum_str!(
    ActionType,
    "ActionType",
    Unspecified => "unspecified",
);

// ---------------------------------------------------------------------------
// Spec-derived semantic helpers for the enums whose meaning is non-trivial.
// ---------------------------------------------------------------------------

impl ErrorLevel {
    /// Per VDA 5050 v3.0.0: whether the mobile robot accepts new orders at
    /// this error level. Fatal blocks new orders; all others accept.
    pub fn accepts_new_orders(self) -> bool {
        !matches!(self, Self::ErrorFatal)
    }

    /// Per VDA 5050 v3.0.0: whether the mobile robot can continue its
    /// active order. Only Critical and Fatal block continuation.
    pub fn can_continue_order(self) -> bool {
        !matches!(self, Self::ErrorCritical | Self::ErrorFatal)
    }
}

impl ConnectionState {
    /// True for terminal-disconnect states.
    pub fn is_disconnected(self) -> bool {
        matches!(self, Self::ConnectionOffline | Self::ConnectionBroken)
    }
}

impl EStopType {
    /// True when no e-stop is active.
    pub fn is_clear(self) -> bool {
        matches!(self, Self::EstopNone)
    }
}

impl MapStatus {
    /// True when the map is currently in use by the mobile robot.
    pub fn is_active(self) -> bool {
        matches!(self, Self::MapEnabled)
    }
}

impl GrantType {
    /// True when the grant allows the mobile robot to proceed.
    pub fn is_positive(self) -> bool {
        matches!(self, Self::GrantGranted | Self::GrantQueued)
    }
}
