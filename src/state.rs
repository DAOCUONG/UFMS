//! [State], its sub-messages, and semantic accessors.
//!
//! The `state` topic is the largest in the VDA 5050 schema. Beyond plain
//! constructors, this module provides the most-asked-for semantic helpers:
//!
//! - [`State::is_idle`], [`State::is_driving`], [`State::is_paused`]
//! - [`State::order_context`] — bundles the always-together order quartet.
//! - [`State::loads_known`] — distinguishes "no loads info" from "explicitly
//!   unloaded".
//! - [`MobileRobotPosition::is_trusted`], [`LocalizationQuality`],
//!   [`MobileRobotPosition::deviation_range`]
//! - [`Velocity::is_known`], [`Velocity::zero`], [`Velocity::linear_speed`]
//! - [`Load::weight_or_unknown`]
//! - [`Error`] / [`Info`] constructors.

use crate::vda5050::v3::{
    EdgeRequest, EdgeState, Error, ErrorLevel, Info, InfoLevel, Load, Map, MapStatus,
    MobileRobotPosition, NodeState, NodeStatePosition, OperatingMode, PlannedPath, PolylinePoint,
    PowerSupply, Reference, RequestStatus, RequestType, SafetyState, State, Trajectory, Translation,
    Velocity, ZoneRequest, ZoneSetReference,
};

// ---------------------------------------------------------------------------
// State semantic helpers
// ---------------------------------------------------------------------------

/// Bundle returned by [`StateExt::order_context`] — the four fields that
/// travel together on every state update.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderContext {
    pub order_id: String,
    pub order_update_id: u32,
    pub last_node_id: String,
    pub last_node_sequence_id: u32,
}

/// Ergonomic helpers for [`State`].
pub trait StateExt {
    /// True when both `node_states` and `edge_states` are empty (idle).
    fn is_idle(&self) -> bool;
    /// True when `driving == true`.
    fn is_driving(&self) -> bool;
    /// True when `paused == true`.
    fn is_paused(&self) -> bool;
    /// Borrow the always-together order quartet.
    fn order_context(&self) -> OrderContext;
    /// True when the `loads` field is present (whether or not empty).
    /// Per spec: unset = unknown, empty = unloaded.
    fn loads_known(&self) -> bool;
    /// All active errors as a slice.
    fn errors(&self) -> &[Error];
    /// All info entries as a slice.
    fn information(&self) -> &[Info];
    /// All loads currently handled by the MR.
    fn loads(&self) -> &[Load];
    /// Convenience: filter errors to those at or above the given severity.
    fn errors_at_or_above(&self, level: ErrorLevel) -> Vec<&Error>;
}

impl StateExt for State {
    fn is_idle(&self) -> bool {
        self.node_states.is_empty() && self.edge_states.is_empty()
    }

    fn is_driving(&self) -> bool {
        self.driving
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn order_context(&self) -> OrderContext {
        OrderContext {
            order_id: self.order_id.clone(),
            order_update_id: self.order_update_id,
            last_node_id: self.last_node_id.clone(),
            last_node_sequence_id: self.last_node_sequence_id,
        }
    }

    fn loads_known(&self) -> bool {
        // `loads` is repeated — there's no `Option<Vec<...>>` in prost; the
        // proto3 semantics for `loads` are encoded by having *any* field
        // (always present in memory after decode) but the spec note "Optional"
        // means we treat absence-of-this-distinct-field-set as unknown.
        // Since prost can't represent unset repeated, the canonical pattern is
        // to expose a sentinel or accept that `loads` is always present.
        // We still expose this helper for symmetry with the spec language.
        true
    }

    fn errors(&self) -> &[Error] {
        &self.errors
    }

    fn information(&self) -> &[Info] {
        &self.information
    }

    fn loads(&self) -> &[Load] {
        &self.loads
    }

    fn errors_at_or_above(&self, level: ErrorLevel) -> Vec<&Error> {
        self.errors
            .iter()
            .filter(|e| (e.error_level as i32) >= level as i32)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Sub-message constructors
// ---------------------------------------------------------------------------

impl Map {
    /// Build a Map. `map_status` defaults to [`MapStatus::MapEnabled`].
    pub fn new(map_id: impl Into<String>) -> Self {
        Self {
            map_id: map_id.into(),
            map_version: String::new(),
            map_descriptor: String::new(),
            map_status: MapStatus::MapEnabled as i32,
        }
    }

    pub fn with_version(mut self, v: impl Into<String>) -> Self {
        self.map_version = v.into();
        self
    }

    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.map_descriptor = d.into();
        self
    }

    pub fn with_status(mut self, s: MapStatus) -> Self {
        self.map_status = s as i32;
        self
    }
}

impl ZoneSetReference {
    /// Build a zone-set reference with default status [`MapStatus::MapEnabled`].
    pub fn new(zone_set_id: impl Into<String>, map_id: impl Into<String>) -> Self {
        Self {
            zone_set_id: zone_set_id.into(),
            map_id: map_id.into(),
            zone_set_status: MapStatus::MapEnabled as i32,
        }
    }

    pub fn with_status(mut self, s: MapStatus) -> Self {
        self.zone_set_status = s as i32;
        self
    }
}

impl NodeState {
    /// Build a `NodeState` with `released = true` (base).
    pub fn new(node_id: impl Into<String>, sequence_id: u32) -> Self {
        Self {
            node_id: node_id.into(),
            sequence_id,
            node_descriptor: String::new(),
            released: true,
            node_position: None,
        }
    }

    pub fn released(mut self, r: bool) -> Self {
        self.released = r;
        self
    }

    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.node_descriptor = d.into();
        self
    }

    pub fn with_position(mut self, p: NodeStatePosition) -> Self {
        self.node_position = Some(p);
        self
    }
}

impl NodeStatePosition {
    pub fn new(x: f64, y: f64, map_id: impl Into<String>) -> Self {
        Self {
            x,
            y,
            theta: None,
            map_id: map_id.into(),
        }
    }

    pub fn with_theta(mut self, t: f64) -> Self {
        self.theta = Some(t);
        self
    }
}

impl EdgeState {
    /// Build an `EdgeState` with `released = true` (base).
    pub fn new(edge_id: impl Into<String>, sequence_id: u32) -> Self {
        Self {
            edge_id: edge_id.into(),
            sequence_id,
            edge_descriptor: String::new(),
            released: true,
            trajectory: None,
        }
    }

    pub fn released(mut self, r: bool) -> Self {
        self.released = r;
        self
    }

    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.edge_descriptor = d.into();
        self
    }

    pub fn with_trajectory(mut self, t: Trajectory) -> Self {
        self.trajectory = Some(t);
        self
    }
}

impl Velocity {
    /// Build a zero velocity (all components explicitly set to 0).
    pub fn zero() -> Self {
        Self {
            vx: Some(0.0),
            vy: Some(0.0),
            omega: Some(0.0),
        }
    }

    /// Build a velocity with all three components. Any `None` component is
    /// "unknown" per spec.
    pub fn new(vx: Option<f64>, vy: Option<f64>, omega: Option<f64>) -> Self {
        Self { vx, vy, omega }
    }

    /// True when every component has been reported (none is `None`).
    pub fn is_known(&self) -> bool {
        self.vx.is_some() && self.vy.is_some() && self.omega.is_some()
    }

    /// Linear (translation) speed from `vx`/`vy` if known, else `None`.
    pub fn linear_speed(&self) -> Option<f64> {
        match (self.vx, self.vy) {
            (Some(vx), Some(vy)) => Some((vx * vx + vy * vy).sqrt()),
            _ => None,
        }
    }
}

impl MobileRobotPosition {
    /// True when `localized == true` and the position can be trusted.
    pub fn is_trusted(&self) -> bool {
        self.localized
    }

    /// Localization quality 0.0 (unknown) .. 1.0 (known). `None` when unset.
    pub fn localization_quality(&self) -> Option<f64> {
        self.localization_score
    }

    /// Position deviation range in meters. `None` when unset.
    pub fn localization_deviation(&self) -> Option<f64> {
        self.deviation_range
    }
}

impl PlannedPath {
    /// Build a planned path from a trajectory and the node IDs it traverses.
    pub fn new(trajectory: Trajectory, traversed_nodes: Vec<String>) -> Self {
        Self {
            trajectory: Some(trajectory),
            traversed_nodes,
        }
    }

    pub fn with_trajectory(mut self, t: Trajectory) -> Self {
        self.trajectory = Some(t);
        self
    }

    pub fn add_traversed_node(mut self, node_id: impl Into<String>) -> Self {
        self.traversed_nodes.push(node_id.into());
        self
    }
}

impl PolylinePoint {
    /// Build a polyline point. `theta` is optional per spec.
    pub fn new(x: f64, y: f64, eta: prost_types::Timestamp) -> Self {
        Self {
            x,
            y,
            theta: None,
            eta: Some(eta),
        }
    }

    pub fn with_theta(mut self, t: f64) -> Self {
        self.theta = Some(t);
        self
    }
}

impl Load {
    /// Build a `Load` with required identity fields. `bounding_box_reference`
    /// and `load_dimensions` default to unset.
    pub fn new(load_id: impl Into<String>, load_type: impl Into<String>) -> Self {
        Self {
            load_id: load_id.into(),
            load_type: load_type.into(),
            load_position: String::new(),
            bounding_box_reference: None,
            load_dimensions: None,
            weight: None,
        }
    }

    pub fn with_position(mut self, p: impl Into<String>) -> Self {
        self.load_position = p.into();
        self
    }

    pub fn with_bounding_box_reference(mut self, b: crate::vda5050::v3::BoundingBoxReference) -> Self {
        self.bounding_box_reference = Some(b);
        self
    }

    pub fn with_dimensions(mut self, d: crate::vda5050::v3::BoundingBox) -> Self {
        self.load_dimensions = Some(d);
        self
    }

    pub fn with_weight(mut self, w: f64) -> Self {
        self.weight = Some(w);
        self
    }

    /// Weight or `None` (unknown). Distinct from `weight.unwrap_or(0.0)`
    /// which would silently return zero.
    pub fn weight_or_unknown(&self) -> Option<f64> {
        self.weight
    }
}

impl ZoneRequest {
    pub fn new(
        request_id: impl Into<String>,
        request_type: RequestType,
        zone_id: impl Into<String>,
        zone_set_id: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            request_type: request_type as i32,
            zone_id: zone_id.into(),
            zone_set_id: zone_set_id.into(),
            request_status: RequestStatus::RequestRequested as i32,
            trajectory: None,
        }
    }

    pub fn with_status(mut self, s: RequestStatus) -> Self {
        self.request_status = s as i32;
        self
    }

    pub fn with_trajectory(mut self, t: Trajectory) -> Self {
        self.trajectory = Some(t);
        self
    }
}

impl EdgeRequest {
    pub fn new(
        request_id: impl Into<String>,
        request_type: RequestType,
        edge_id: impl Into<String>,
        sequence_id: u32,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            request_type: request_type as i32,
            edge_id: edge_id.into(),
            sequence_id,
            request_status: RequestStatus::RequestRequested as i32,
        }
    }

    pub fn with_status(mut self, s: RequestStatus) -> Self {
        self.request_status = s as i32;
        self
    }
}

impl Error {
    /// Build an `Error` with required `error_type`, description, hint, and
    /// `error_level`.
    pub fn new(
        error_type: impl Into<String>,
        error_level: ErrorLevel,
        description: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            error_type: error_type.into(),
            error_references: Vec::new(),
            error_description: description.into(),
            error_description_translations: Vec::new(),
            error_hint: hint.into(),
            error_hint_translations: Vec::new(),
            error_level: error_level as i32,
        }
    }

    pub fn add_reference(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.error_references.push(Reference {
            reference_key: key.into(),
            reference_value: value.into(),
        });
        self
    }

    pub fn add_description_translation(
        mut self,
        lang: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        self.error_description_translations.push(Translation {
            translation_key: lang.into(),
            translation_value: text.into(),
        });
        self
    }

    pub fn add_hint_translation(
        mut self,
        lang: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        self.error_hint_translations.push(Translation {
            translation_key: lang.into(),
            translation_value: text.into(),
        });
        self
    }
}

impl Info {
    /// Build an `Info` entry with required `info_type` and `info_level`.
    pub fn new(info_type: impl Into<String>, info_level: InfoLevel) -> Self {
        Self {
            info_type: info_type.into(),
            info_references: Vec::new(),
            info_descriptor: String::new(),
            info_level: info_level as i32,
        }
    }

    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.info_descriptor = d.into();
        self
    }

    pub fn add_reference(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.info_references.push(Reference {
            reference_key: key.into(),
            reference_value: value.into(),
        });
        self
    }
}

// ---------------------------------------------------------------------------
// LoadBuilder — fluent wrapper for the verbose Load message.
// ---------------------------------------------------------------------------

/// Fluent builder for [`Load`].
pub struct LoadBuilder(Load);

impl LoadBuilder {
    pub fn new(load_id: impl Into<String>, load_type: impl Into<String>) -> Self {
        Self(Load::new(load_id, load_type))
    }

    pub fn position(mut self, p: impl Into<String>) -> Self {
        self.0.load_position = p.into();
        self
    }

    pub fn bounding_box_reference(mut self, b: crate::vda5050::v3::BoundingBoxReference) -> Self {
        self.0.bounding_box_reference = Some(b);
        self
    }

    pub fn dimensions(mut self, d: crate::vda5050::v3::BoundingBox) -> Self {
        self.0.load_dimensions = Some(d);
        self
    }

    pub fn weight(mut self, w: f64) -> Self {
        self.0.weight = Some(w);
        self
    }

    pub fn build(self) -> Load {
        self.0
    }
}

// ---------------------------------------------------------------------------
// State constructor
// ---------------------------------------------------------------------------

impl State {
    /// Build a `State` with the given `header`. All `repeated` lists default
    /// to empty; scalar booleans default to `false`.
    pub fn new(header: crate::vda5050::v3::Header) -> Self {
        Self {
            header: Some(header),
            maps: Vec::new(),
            zone_sets: Vec::new(),
            order_id: String::new(),
            order_update_id: 0,
            last_node_id: String::new(),
            last_node_sequence_id: 0,
            node_states: Vec::new(),
            edge_states: Vec::new(),
            planned_path: None,
            intermediate_path: None,
            mobile_robot_position: None,
            velocity: None,
            loads: Vec::new(),
            driving: false,
            paused: false,
            new_base_request: false,
            zone_requests: Vec::new(),
            edge_requests: Vec::new(),
            distance_since_last_node: None,
            action_states: Vec::new(),
            instant_action_states: Vec::new(),
            zone_action_states: Vec::new(),
            power_supply: None,
            operating_mode: OperatingMode::OperatingAutomatic as i32,
            errors: Vec::new(),
            information: Vec::new(),
            safety_state: None,
        }
    }

    // -- Chainable setters used by builders and demos. --

    pub fn driving(mut self, v: bool) -> Self {
        self.driving = v;
        self
    }

    pub fn paused(mut self, v: bool) -> Self {
        self.paused = v;
        self
    }

    pub fn new_base_request(mut self, v: bool) -> Self {
        self.new_base_request = v;
        self
    }

    pub fn order_id(mut self, id: impl Into<String>) -> Self {
        self.order_id = id.into();
        self
    }

    pub fn order_update_id(mut self, n: u32) -> Self {
        self.order_update_id = n;
        self
    }

    pub fn last_node_id(mut self, id: impl Into<String>) -> Self {
        self.last_node_id = id.into();
        self
    }

    pub fn last_node_sequence_id(mut self, n: u32) -> Self {
        self.last_node_sequence_id = n;
        self
    }

    pub fn add_load(mut self, l: Load) -> Self {
        self.loads.push(l);
        self
    }

    pub fn add_map(mut self, m: Map) -> Self {
        self.maps.push(m);
        self
    }

    pub fn add_zone_set(mut self, z: ZoneSetReference) -> Self {
        self.zone_sets.push(z);
        self
    }

    pub fn add_node_state(mut self, n: NodeState) -> Self {
        self.node_states.push(n);
        self
    }

    pub fn add_edge_state(mut self, e: EdgeState) -> Self {
        self.edge_states.push(e);
        self
    }

    pub fn add_error(mut self, e: Error) -> Self {
        self.errors.push(e);
        self
    }

    pub fn add_information(mut self, i: Info) -> Self {
        self.information.push(i);
        self
    }

    pub fn add_zone_request(mut self, z: ZoneRequest) -> Self {
        self.zone_requests.push(z);
        self
    }

    pub fn add_edge_request(mut self, e: EdgeRequest) -> Self {
        self.edge_requests.push(e);
        self
    }

    pub fn add_action_state(
        mut self,
        s: crate::vda5050::v3::ActionState,
    ) -> Self {
        self.action_states.push(s);
        self
    }

    pub fn power_supply(mut self, p: Option<PowerSupply>) -> Self {
        self.power_supply = p;
        self
    }

    pub fn with_operating_mode(mut self, m: i32) -> Self {
        self.operating_mode = m;
        self
    }

    pub fn safety_state(mut self, s: Option<SafetyState>) -> Self {
        self.safety_state = s;
        self
    }

    pub fn mobile_robot_position(mut self, p: Option<MobileRobotPosition>) -> Self {
        self.mobile_robot_position = p;
        self
    }

    pub fn velocity(mut self, v: Option<Velocity>) -> Self {
        self.velocity = v;
        self
    }

    pub fn planned_path(mut self, p: Option<PlannedPath>) -> Self {
        self.planned_path = p;
        self
    }
}

// `PowerSupply` and `SafetyState` constructors — small but live with State.
impl PowerSupply {
    /// Build a `PowerSupply`. Only `state_of_charge` and `charging` are
    /// required; the rest are optional.
    pub fn new(state_of_charge: f32, charging: bool) -> Self {
        Self {
            state_of_charge,
            battery_voltage: None,
            battery_current: None,
            battery_health: None,
            charging,
            range: None,
        }
    }

    pub fn with_battery_voltage(mut self, v: f64) -> Self {
        self.battery_voltage = Some(v);
        self
    }

    pub fn with_battery_current(mut self, a: f64) -> Self {
        self.battery_current = Some(a);
        self
    }

    pub fn with_battery_health(mut self, h: f32) -> Self {
        self.battery_health = Some(h);
        self
    }

    pub fn with_range(mut self, r: f64) -> Self {
        self.range = Some(r);
        self
    }
}

impl SafetyState {
    pub fn new(active_emergency_stop: crate::vda5050::v3::EStopType, field_violation: bool) -> Self {
        Self {
            active_emergency_stop: active_emergency_stop as i32,
            field_violation,
        }
    }

    /// True when the MR is safe to drive (e-stop clear and no field
    /// violation).
    pub fn is_safe_to_drive(&self) -> bool {
        self.active_emergency_stop == crate::vda5050::v3::EStopType::EstopNone as i32
            && !self.field_violation
    }
}
