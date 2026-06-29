//! UFMS — UFleet Management System.
//!
//! Generated VDA 5050 v3.0.0 protobuf types are re-exported under the
//! `vda5050::v3` module tree (via `prost-build` in `build.rs`). On top of
//! the generated types we provide a hand-written ergonomic layer with:
//!
//! - type-safe enum helpers ([`vda5050::v3::EnumExt`]),
//! - uniform [`Header`](crate::header::Header) construction and the
//!   [`HeaderExt`](crate::header::HeaderExt) trait for every header-bearing
//!   top-level message,
//! - per-topic constructors, builders, and validators,
//! - geometric helpers ([`Vertex2D`](crate::geometry::Vertex2D),
//!   [`polygon_area`](crate::geometry::polygon_area), etc.),
//! - and the `Action` builder that forces the spec rules
//!   `instantActions → BlockingNone` and so on.

pub mod vda5050 {
    pub mod v3 {
        include!(concat!(env!("OUT_DIR"), "/vda5050.v3.rs"));
    }
}

// Re-export the ergonomic layer at the crate root for short access paths.
pub mod action;
pub mod connection;
pub mod enums;
pub mod error;
pub mod factsheet;
pub mod geometry;
pub mod header;
pub mod instant_actions;
pub mod order;
pub mod responses;
pub mod state;
pub mod visualization;
pub mod zone_set;

/// Convenience prelude: `use ufms::prelude::*;` brings the generated
/// `vda5050::v3` types plus the ergonomic helper traits into scope.
pub mod prelude {
    pub use crate::action::ActionBuilder;
    pub use crate::connection::ConnectionExt;
    pub use crate::enums::EnumExt;
    pub use crate::error::{
        ActionError as ActionValidationError, HeaderError, TimestampError, UnknownVariant,
        ValidationError,
    };
    pub use crate::geometry::{
        point_in_polygon, polygon_area, polygon_area_unsigned, polygon_bounding_box,
        polygon_centroid, polygon_perimeter,
    };
    pub use crate::header::{HeaderExt, TimestampExt};
    pub use crate::instant_actions::InstantActionsBuilder;
    pub use crate::order::{CorridorExt, EdgeBuilder, NodeBuilder, OrderBuilder, OrderExt};
    pub use crate::responses::{ResponseBuilder, ResponsesExt};
    pub use crate::state::{LoadBuilder, StateExt};
    pub use crate::vda5050::v3::*;
    pub use crate::visualization::VisualizationBuilder;
    pub use crate::zone_set::{ZoneBuilder, ZoneSetBuilder, ZoneSetExt};
}
