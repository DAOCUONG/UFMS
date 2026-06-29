//! [ZoneSet] and per-zone-type [Zone] constructors.
//!
//! The proto schema encodes a conditional `if/then` table for zone fields
//! (see `zone_set.proto` file-level comment). [`Zone`] constructors below
//! force the right combination of fields for each `ZoneType` so callers
//! can't forget the conditional fields, and [`Zone::validate`] /
//! [`ZoneSetExt::validate`] catch construction mistakes after the fact
//! (e.g., when decoding incoming bytes).

use crate::error::ValidationError;
use crate::geometry::{point_in_polygon, polygon_area};
use crate::vda5050::v3::{
    BidirectedLimitation, BlockingType, DirectedLimitation, Header, ReleaseLossBehavior, Vertex2D,
    Zone, ZoneAction, ZoneSet, ZoneSetData, ZoneType,
};

// ---------------------------------------------------------------------------
// Zone per-zone-type constructors
// ---------------------------------------------------------------------------

impl Zone {
    /// Helper that asserts a polygonal-vertex invariant (≥3 vertices) and
    /// returns a new [`Zone`] with the common fields populated.
    fn skeleton(
        zone_id: impl Into<String>,
        zone_type: ZoneType,
        descriptor: impl Into<String>,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        Self {
            zone_id: zone_id.into(),
            zone_type: zone_type as i32,
            zone_descriptor: descriptor.into(),
            vertices,
            release_loss_behavior: None,
            maximum_speed: None,
            entry_actions: Vec::new(),
            during_actions: Vec::new(),
            exit_actions: Vec::new(),
            priority_factor: None,
            penalty_factor: None,
            direction: None,
            directed_limitation: None,
            bidirected_direction: None,
            bidirected_limitation: None,
        }
    }

    fn require_polygon(context: &'static str, vertices: &[Vertex2D]) -> Result<(), ValidationError> {
        if vertices.len() < 3 {
            return Err(ValidationError::PolygonTooSmall {
                context,
                len: vertices.len(),
            });
        }
        Ok(())
    }

    /// Build a `ZoneBlocked` zone. No conditional fields.
    pub fn blocked(zone_id: impl Into<String>, vertices: Vec<Vertex2D>) -> Self {
        Self::skeleton(zone_id, ZoneType::ZoneBlocked, "", vertices)
    }

    /// Build a `ZoneLineGuided` zone. No conditional fields.
    pub fn line_guided(zone_id: impl Into<String>, vertices: Vec<Vertex2D>) -> Self {
        Self::skeleton(zone_id, ZoneType::ZoneLineGuided, "", vertices)
    }

    /// Build a `ZoneCoordinatedReplanning` zone. No conditional fields.
    pub fn coordinated_replanning(zone_id: impl Into<String>, vertices: Vec<Vertex2D>) -> Self {
        Self::skeleton(
            zone_id,
            ZoneType::ZoneCoordinatedReplanning,
            "",
            vertices,
        )
    }

    /// Build a `ZoneRelease` zone. `release_loss_behavior` is required by
    /// the spec.
    pub fn release(
        zone_id: impl Into<String>,
        release_loss_behavior: ReleaseLossBehavior,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZoneRelease, "", vertices);
        z.release_loss_behavior = Some(release_loss_behavior as i32);
        z
    }

    /// Build a `ZoneSpeedLimit` zone. `maximum_speed` is required by the
    /// spec.
    pub fn speed_limit(
        zone_id: impl Into<String>,
        maximum_speed: f64,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZoneSpeedLimit, "", vertices);
        z.maximum_speed = Some(maximum_speed);
        z
    }

    /// Build a `ZoneAction` zone. `entry_actions`/`during_actions`/
    /// `exit_actions` are required by the spec (they may be empty lists).
    pub fn action(
        zone_id: impl Into<String>,
        entry_actions: Vec<ZoneAction>,
        during_actions: Vec<ZoneAction>,
        exit_actions: Vec<ZoneAction>,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZoneAction, "", vertices);
        z.entry_actions = entry_actions;
        z.during_actions = during_actions;
        z.exit_actions = exit_actions;
        z
    }

    /// Build a `ZonePriority` zone. `priority_factor` is required by the
    /// spec (range 0.0..1.0).
    pub fn priority(
        zone_id: impl Into<String>,
        priority_factor: f64,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZonePriority, "", vertices);
        z.priority_factor = Some(priority_factor);
        z
    }

    /// Build a `ZonePenalty` zone. `penalty_factor` is required by the spec
    /// (range 0.0..1.0).
    pub fn penalty(
        zone_id: impl Into<String>,
        penalty_factor: f64,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZonePenalty, "", vertices);
        z.penalty_factor = Some(penalty_factor);
        z
    }

    /// Build a `ZoneDirected` zone. `direction` + `directed_limitation` are
    /// required by the spec.
    pub fn directed(
        zone_id: impl Into<String>,
        direction_rad: f64,
        directed_limitation: DirectedLimitation,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZoneDirected, "", vertices);
        z.direction = Some(direction_rad);
        z.directed_limitation = Some(directed_limitation as i32);
        z
    }

    /// Build a `ZoneBidirected` zone. `bidirected_direction` (and its
    /// opposite `+ PI`) and `bidirected_limitation` are required by the
    /// spec.
    pub fn bidirected(
        zone_id: impl Into<String>,
        bidirected_direction_rad: f64,
        bidirected_limitation: BidirectedLimitation,
        vertices: Vec<Vertex2D>,
    ) -> Self {
        let mut z = Self::skeleton(zone_id, ZoneType::ZoneBidirected, "", vertices);
        z.bidirected_direction = Some(bidirected_direction_rad);
        z.bidirected_limitation = Some(bidirected_limitation as i32);
        z
    }

    /// Chainable setter for `zone_descriptor`.
    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.zone_descriptor = d.into();
        self
    }

    /// Validate a `Zone` against the spec's if/then table. Use this to
    /// double-check decoded messages.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Common invariants — zone_id, zone_type, descriptor, vertices.
        if self.zone_id.is_empty() {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "zone_id",
            });
        }
        Self::require_polygon("Zone.vertices", &self.vertices)?;

        let zt = self.zone_type;
        if zt == ZoneType::ZoneSpeedLimit as i32 && self.maximum_speed.is_none() {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "maximum_speed",
            });
        }
        if zt == ZoneType::ZoneRelease as i32 && self.release_loss_behavior.is_none() {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "release_loss_behavior",
            });
        }
        if zt == ZoneType::ZoneAction as i32
            && (self.entry_actions.is_empty()
                && self.during_actions.is_empty()
                && self.exit_actions.is_empty())
        {
            // Spec says "required when ZONE_ACTION" — we require at least
            // one list be non-empty when classifying as an action zone.
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "entry_actions|during_actions|exit_actions",
            });
        }
        if zt == ZoneType::ZonePriority as i32 && self.priority_factor.is_none() {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "priority_factor",
            });
        }
        if zt == ZoneType::ZonePenalty as i32 && self.penalty_factor.is_none() {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "penalty_factor",
            });
        }
        if zt == ZoneType::ZoneDirected as i32
            && (self.direction.is_none() || self.directed_limitation.is_none())
        {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "direction|directed_limitation",
            });
        }
        if zt == ZoneType::ZoneBidirected as i32
            && (self.bidirected_direction.is_none() || self.bidirected_limitation.is_none())
        {
            return Err(ValidationError::ZoneMissingField {
                zone_id: self.zone_id.clone(),
                field: "bidirected_direction|bidirected_limitation",
            });
        }
        Ok(())
    }

    /// Geometric helper: does the (closed) zone contain this point?
    pub fn contains(&self, point: Vertex2D) -> bool {
        point_in_polygon(point, &self.vertices)
    }

    /// Geometric helper: signed area (positive = counter-clockwise).
    pub fn area(&self) -> f64 {
        polygon_area(&self.vertices)
    }
}

// ---------------------------------------------------------------------------
// ZoneAction constructors
// ---------------------------------------------------------------------------

impl ZoneAction {
    /// Build a `ZoneAction` with the minimum required fields.
    pub fn new(action_type: impl Into<String>, blocking_type: BlockingType) -> Self {
        Self {
            action_type: action_type.into(),
            action_descriptor: String::new(),
            blocking_type: blocking_type as i32,
            action_parameters: Vec::new(),
            retriable: None,
        }
    }

    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.action_descriptor = d.into();
        self
    }

    pub fn with_retriable(mut self, r: bool) -> Self {
        self.retriable = Some(r);
        self
    }

    pub fn add_parameter(mut self, p: crate::vda5050::v3::ActionParameter) -> Self {
        self.action_parameters.push(p);
        self
    }
}

// ---------------------------------------------------------------------------
// ZoneSet / ZoneSetData constructors
// ---------------------------------------------------------------------------

/// Ergonomic helpers for [`ZoneSet`].
pub trait ZoneSetExt {
    /// Validate the entire zone set.
    fn validate(&self) -> Result<(), ValidationError>;
    /// Total number of zones across all sets.
    fn zone_count(&self) -> usize;
}

impl ZoneSetExt for ZoneSet {
    fn validate(&self) -> Result<(), ValidationError> {
        let data = self.zone_set.as_ref().ok_or(ValidationError::ZoneMissingField {
            zone_id: "<unset>".to_string(),
            field: "zone_set",
        })?;
        if data.zone_set_id.is_empty() {
            return Err(ValidationError::ZoneMissingField {
                zone_id: "<unset>".to_string(),
                field: "zone_set.zone_set_id",
            });
        }
        for z in &data.zones {
            z.validate()?;
        }
        Ok(())
    }

    fn zone_count(&self) -> usize {
        self.zone_set
            .as_ref()
            .map(|d| d.zones.len())
            .unwrap_or(0)
    }
}

impl ZoneSet {
    /// Build a new `ZoneSet` referencing `Header`, `map_id`, and
    /// `zone_set_id`. No zones initially.
    pub fn new(header: Header, map_id: impl Into<String>, zone_set_id: impl Into<String>) -> Self {
        Self {
            header: Some(header),
            zone_set: Some(ZoneSetData {
                map_id: map_id.into(),
                zone_set_id: zone_set_id.into(),
                zone_set_descriptor: String::new(),
                zones: Vec::new(),
            }),
        }
    }
}

impl ZoneSetData {
    /// Build a `ZoneSetData` payload (without the outer `Header`).
    pub fn new(map_id: impl Into<String>, zone_set_id: impl Into<String>) -> Self {
        Self {
            map_id: map_id.into(),
            zone_set_id: zone_set_id.into(),
            zone_set_descriptor: String::new(),
            zones: Vec::new(),
        }
    }

    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.zone_set_descriptor = d.into();
        self
    }

    pub fn add_zone(mut self, z: Zone) -> Self {
        self.zones.push(z);
        self
    }
}

// ---------------------------------------------------------------------------
// ZoneBuilder — fluent wrapper for the verbose Zone message.
// ---------------------------------------------------------------------------

/// Fluent builder for [`Zone`]. Start by selecting the zone type via the
/// [`ZoneBuilder::typed`] constructor, then call chainable setters.
pub struct ZoneBuilder(Zone);

impl ZoneBuilder {
    /// Build a builder from an existing [`Zone`].
    pub fn new(z: Zone) -> Self {
        Self(z)
    }

    /// Convenience: build a builder from one of the typed constructors.
    pub fn typed(zone_type: ZoneType, zone_id: impl Into<String>, vertices: Vec<Vertex2D>) -> Self {
        let z = match zone_type {
            ZoneType::ZoneBlocked => Zone::blocked(zone_id, vertices),
            ZoneType::ZoneLineGuided => Zone::line_guided(zone_id, vertices),
            ZoneType::ZoneCoordinatedReplanning => Zone::coordinated_replanning(zone_id, vertices),
            _ => Zone::skeleton(zone_id, zone_type, "", vertices),
        };
        Self(z)
    }

    pub fn descriptor(mut self, d: impl Into<String>) -> Self {
        self.0.zone_descriptor = d.into();
        self
    }

    pub fn maximum_speed(mut self, m: f64) -> Self {
        self.0.maximum_speed = Some(m);
        self
    }

    pub fn release_loss_behavior(mut self, r: ReleaseLossBehavior) -> Self {
        self.0.release_loss_behavior = Some(r as i32);
        self
    }

    pub fn entry_actions(mut self, a: Vec<ZoneAction>) -> Self {
        self.0.entry_actions = a;
        self
    }

    pub fn during_actions(mut self, a: Vec<ZoneAction>) -> Self {
        self.0.during_actions = a;
        self
    }

    pub fn exit_actions(mut self, a: Vec<ZoneAction>) -> Self {
        self.0.exit_actions = a;
        self
    }

    pub fn priority_factor(mut self, p: f64) -> Self {
        self.0.priority_factor = Some(p);
        self
    }

    pub fn penalty_factor(mut self, p: f64) -> Self {
        self.0.penalty_factor = Some(p);
        self
    }

    pub fn direction(mut self, d: f64) -> Self {
        self.0.direction = Some(d);
        self
    }

    pub fn directed_limitation(mut self, d: DirectedLimitation) -> Self {
        self.0.directed_limitation = Some(d as i32);
        self
    }

    pub fn bidirected_direction(mut self, d: f64) -> Self {
        self.0.bidirected_direction = Some(d);
        self
    }

    pub fn bidirected_limitation(mut self, b: BidirectedLimitation) -> Self {
        self.0.bidirected_limitation = Some(b as i32);
        self
    }

    /// Validate before building. Use [`build_unchecked`] if you have already
    /// validated externally.
    pub fn try_build(self) -> Result<Zone, ValidationError> {
        self.0.validate()?;
        Ok(self.0)
    }

    pub fn build_unchecked(self) -> Zone {
        self.0
    }

    pub fn build(self) -> Zone {
        self.0
    }
}

/// Fluent builder for [`ZoneSet`].
pub struct ZoneSetBuilder(ZoneSet);

impl ZoneSetBuilder {
    pub fn new(header: Header, map_id: impl Into<String>, zone_set_id: impl Into<String>) -> Self {
        Self(ZoneSet::new(header, map_id, zone_set_id))
    }

    pub fn descriptor(mut self, d: impl Into<String>) -> Self {
        if let Some(data) = self.0.zone_set.as_mut() {
            data.zone_set_descriptor = d.into();
        }
        self
    }

    pub fn add_zone(mut self, z: Zone) -> Self {
        if let Some(data) = self.0.zone_set.as_mut() {
            data.zones.push(z);
        }
        self
    }

    pub fn try_build(self) -> Result<ZoneSet, ValidationError> {
        self.0.validate()?;
        Ok(self.0)
    }

    pub fn build(self) -> ZoneSet {
        self.0
    }
}
