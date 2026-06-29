//! [Factsheet] constructors and helpers for its sub-sections.
//!
//! The factsheet topic is the largest in the VDA 5050 schema (392 lines,
//! seven sub-sections). This module adds ergonomic constructors and
//! chainable paired setters for the most commonly-co-occurring fields; any
//! field not covered here can still be set via direct field assignment.

use prost_types::Struct;

use crate::error::ValidationError;
use crate::geometry::{polygon_area, polygon_perimeter};
use crate::vda5050::v3::{
    ActionParameterDescription, ActionScope, BatteryCharging, BlockingType, BoundingBox,
    BoundingBoxReference, Envelope2D, Envelope3D, Factsheet, Header, LoadSet, LoadSpecification,
    MaximumArrayLengths, MaximumStringLengths, MobileRobotAction, MobileRobotConfiguration,
    MobileRobotGeometry, Network, OptionalParameter, OptionalParameterSupport, PhysicalParameters,
    ProtocolFeatures, ProtocolLimits, Timing, TypeSpecification, ValueDataType, Version,
    WheelDefinition, WheelPosition, ZoneType,
};

// ---------------------------------------------------------------------------
// Factsheet
// ---------------------------------------------------------------------------

impl Factsheet {
    /// Build a factsheet with the given identity. All seven sub-sections
    /// default to `None`; populate them with the chained setters.
    pub fn new(series_name: impl Into<String>, series_description: impl Into<String>) -> Self {
        let ts = TypeSpecification {
            series_name: series_name.into(),
            series_description: series_description.into(),
            ..Default::default()
        };
        Self {
            header: None,
            type_specification: Some(ts),
            physical_parameters: None,
            protocol_limits: None,
            protocol_features: None,
            mobile_robot_geometry: None,
            load_specification: None,
            mobile_robot_configuration: None,
        }
    }

    pub fn with_header(mut self, h: Header) -> Self {
        self.header = Some(h);
        self
    }

    pub fn with_physical_parameters(mut self, p: PhysicalParameters) -> Self {
        self.physical_parameters = Some(p);
        self
    }

    pub fn with_protocol_limits(mut self, p: ProtocolLimits) -> Self {
        self.protocol_limits = Some(p);
        self
    }

    pub fn with_protocol_features(mut self, p: ProtocolFeatures) -> Self {
        self.protocol_features = Some(p);
        self
    }

    pub fn with_mobile_robot_geometry(mut self, g: MobileRobotGeometry) -> Self {
        self.mobile_robot_geometry = Some(g);
        self
    }

    pub fn with_load_specification(mut self, l: LoadSpecification) -> Self {
        self.load_specification = Some(l);
        self
    }

    pub fn with_mobile_robot_configuration(mut self, c: MobileRobotConfiguration) -> Self {
        self.mobile_robot_configuration = Some(c);
        self
    }
}

// `TypeSpecification` already has `Default` from `prost::Message`. We just
// need a real constructor.

impl TypeSpecification {
    /// Build a TypeSpecification with the three identifying fields.
    pub fn new(
        series_name: impl Into<String>,
        series_description: impl Into<String>,
        mobile_robot_class: impl Into<String>,
        mobile_robot_kinematics: impl Into<String>,
    ) -> Self {
        Self {
            series_name: series_name.into(),
            series_description: series_description.into(),
            mobile_robot_kinematics: mobile_robot_kinematics.into(),
            mobile_robot_class: mobile_robot_class.into(),
            maximum_load_mass: 0.0,
            localization_types: Vec::new(),
            navigation_types: Vec::new(),
            supported_zones: Vec::new(),
        }
    }

    pub fn with_maximum_load_mass(mut self, kg: f64) -> Self {
        self.maximum_load_mass = kg;
        self
    }

    pub fn add_localization_type(mut self, t: impl Into<String>) -> Self {
        self.localization_types.push(t.into());
        self
    }

    pub fn add_navigation_type(mut self, t: impl Into<String>) -> Self {
        self.navigation_types.push(t.into());
        self
    }

    pub fn add_supported_zone(mut self, z: ZoneType) -> Self {
        self.supported_zones.push(z as i32);
        self
    }
}

// ---------------------------------------------------------------------------
// PhysicalParameters — paired setters
// ---------------------------------------------------------------------------

impl PhysicalParameters {
    /// Build with mandatory translational kinematics, acceleration, height
    /// range, width, length. Angles default to unset.
    pub fn new(
        min_speed: f64,
        max_speed: f64,
        max_acceleration: f64,
        max_deceleration: f64,
        min_height: f64,
        max_height: f64,
        width: f64,
        length: f64,
    ) -> Self {
        Self {
            minimum_speed: min_speed,
            maximum_speed: max_speed,
            minimum_angular_speed: None,
            maximum_angular_speed: None,
            maximum_acceleration: max_acceleration,
            maximum_deceleration: max_deceleration,
            minimum_height: min_height,
            maximum_height: max_height,
            width,
            length,
        }
    }

    pub fn with_angular(mut self, min: f64, max: f64) -> Self {
        self.minimum_angular_speed = Some(min);
        self.maximum_angular_speed = Some(max);
        self
    }
}

// ---------------------------------------------------------------------------
// ProtocolLimits sub-types
// ---------------------------------------------------------------------------

impl Timing {
    pub fn new(min_order_interval: f64, min_state_interval: f64) -> Self {
        Self {
            minimum_order_interval: min_order_interval,
            minimum_state_interval: min_state_interval,
            default_state_interval: None,
            visualization_interval: None,
        }
    }

    pub fn with_default_state_interval(mut self, s: f64) -> Self {
        self.default_state_interval = Some(s);
        self
    }

    pub fn with_visualization_interval(mut self, s: f64) -> Self {
        self.visualization_interval = Some(s);
        self
    }
}

impl MaximumStringLengths {
    /// All-unset (no explicit limits).
    pub fn unbounded() -> Self {
        Self {
            maximum_message_length: None,
            maximum_topic_serial_length: None,
            maximum_topic_element_length: None,
            maximum_id_length: None,
            id_numerical_only: None,
            maximum_load_id_length: None,
        }
    }

    pub fn with_ids(mut self, max_len: u32, numerical_only: bool) -> Self {
        self.maximum_id_length = Some(max_len);
        self.id_numerical_only = Some(numerical_only);
        self
    }
}

impl MaximumArrayLengths {
    /// All-unset (no explicit limits — the spec sentinel).
    pub fn unbounded() -> Self {
        Self {
            order_nodes: None,
            order_edges: None,
            node_actions: None,
            edge_actions: None,
            actions_action_parameters: None,
            instant_actions: None,
            trajectory_knot_vector: None,
            trajectory_control_points: None,
            zone_set_zones: None,
            state_node_states: None,
            state_edge_states: None,
            state_loads: None,
            state_action_states: None,
            state_instant_action_states: None,
            state_zone_action_states: None,
            state_errors: None,
            state_information: None,
            error_error_references: None,
            information_info_references: None,
        }
    }
}

impl ProtocolLimits {
    pub fn new(
        string_lengths: MaximumStringLengths,
        array_lengths: MaximumArrayLengths,
        timing: Timing,
    ) -> Self {
        Self {
            maximum_string_lengths: Some(string_lengths),
            maximum_array_lengths: Some(array_lengths),
            timing: Some(timing),
        }
    }
}

// ---------------------------------------------------------------------------
// ProtocolFeatures
// ---------------------------------------------------------------------------

impl OptionalParameter {
    pub fn new(
        parameter: impl Into<String>,
        support: OptionalParameterSupport,
        description: impl Into<String>,
    ) -> Self {
        Self {
            parameter: parameter.into(),
            support: support as i32,
            description: description.into(),
        }
    }
}

impl ActionParameterDescription {
    pub fn new(key: impl Into<String>, value_data_type: ValueDataType) -> Self {
        Self {
            key: key.into(),
            value_data_type: value_data_type as i32,
            description: String::new(),
            is_optional: None,
        }
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    pub fn as_optional(mut self, optional: bool) -> Self {
        self.is_optional = Some(optional);
        self
    }
}

impl MobileRobotAction {
    /// Build an action descriptor for the factsheet. `action_type` is the
    /// well-known name (e.g., `"pick"`, `"drop"`, `"pauseOrder"`).
    pub fn new(action_type: impl Into<String>) -> Self {
        Self {
            action_type: action_type.into(),
            action_description: String::new(),
            action_scopes: Vec::new(),
            action_parameters: Vec::new(),
            action_result: String::new(),
            blocking_types: Vec::new(),
            pause_allowed: false,
            cancel_allowed: false,
        }
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.action_description = d.into();
        self
    }

    pub fn add_scope(mut self, s: ActionScope) -> Self {
        self.action_scopes.push(s as i32);
        self
    }

    pub fn add_blocking_type(mut self, b: BlockingType) -> Self {
        self.blocking_types.push(b as i32);
        self
    }

    pub fn add_parameter(mut self, p: ActionParameterDescription) -> Self {
        self.action_parameters.push(p);
        self
    }

    pub fn action_result(mut self, r: impl Into<String>) -> Self {
        self.action_result = r.into();
        self
    }

    pub fn pause_allowed(mut self, v: bool) -> Self {
        self.pause_allowed = v;
        self
    }

    pub fn cancel_allowed(mut self, v: bool) -> Self {
        self.cancel_allowed = v;
        self
    }
}

impl ProtocolFeatures {
    pub fn new() -> Self {
        Self {
            optional_parameters: Vec::new(),
            mobile_robot_actions: Vec::new(),
        }
    }

    pub fn add_optional_parameter(mut self, p: OptionalParameter) -> Self {
        self.optional_parameters.push(p);
        self
    }

    pub fn add_mobile_robot_action(mut self, a: MobileRobotAction) -> Self {
        self.mobile_robot_actions.push(a);
        self
    }
}

// ---------------------------------------------------------------------------
// MobileRobotGeometry
// ---------------------------------------------------------------------------

impl WheelPosition {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            theta: None,
        }
    }

    pub fn with_theta(mut self, theta: f64) -> Self {
        self.theta = Some(theta);
        self
    }
}

impl WheelDefinition {
    /// Build a wheel with the four required fields. `wheel_type` is a
    /// free-text extensible enum (e.g., `"DRIVE"`, `"CASTER"`, `"FIXED"`,
    /// `"MECANUM"`).
    pub fn new(
        wheel_type: impl Into<String>,
        diameter: f64,
        width: f64,
        position: WheelPosition,
    ) -> Self {
        Self {
            r#type: wheel_type.into(),
            is_active_driven: false,
            is_active_steered: false,
            position: Some(position),
            diameter,
            width,
            center_displacement: None,
            constraints: String::new(),
        }
    }

    pub fn fixed_wheel(x: f64, y: f64, theta: f64, diameter: f64, width: f64) -> Self {
        Self::new("FIXED", diameter, width, WheelPosition::new(x, y).with_theta(theta))
    }

    pub fn caster_wheel(
        x: f64,
        y: f64,
        diameter: f64,
        width: f64,
        center_displacement: f64,
    ) -> Self {
        let mut w = Self::new("CASTER", diameter, width, WheelPosition::new(x, y));
        w.center_displacement = Some(center_displacement);
        w
    }

    pub fn mecanum_wheel(x: f64, y: f64, diameter: f64, width: f64) -> Self {
        Self::new("MECANUM", diameter, width, WheelPosition::new(x, y))
    }

    pub fn driven_active(mut self) -> Self {
        self.is_active_driven = true;
        self
    }

    pub fn steered_active(mut self) -> Self {
        self.is_active_steered = true;
        self
    }
}

impl Envelope2D {
    /// Build an `Envelope2D`. Validates that vertices contain ≥ 3 points
    /// (per the `polygon` invariant).
    pub fn new(id: impl Into<String>, vertices: Vec<crate::vda5050::v3::Vertex2D>) -> Self {
        Self {
            envelope2d_id: id.into(),
            vertices,
            description: String::new(),
        }
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    /// Total 2D area of the polygon, in m².
    pub fn area(&self) -> f64 {
        polygon_area_unsigned_safe(&self.vertices)
    }

    /// Total perimeter of the polygon, in m.
    pub fn perimeter(&self) -> f64 {
        polygon_perimeter(&self.vertices)
    }
}

fn polygon_area_unsigned_safe(v: &[crate::vda5050::v3::Vertex2D]) -> f64 {
    polygon_area(v).abs()
}

impl Envelope3D {
    /// Build an `Envelope3D` carrying a `url` (mutually-exclusive-with-data
    /// invariant).
    pub fn with_url(
        id: impl Into<String>,
        format: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            envelope3d_id: id.into(),
            format: format.into(),
            data: None,
            url: url.into(),
            description: String::new(),
        }
    }

    /// Build an `Envelope3D` carrying inline `data` (mutually-exclusive-with-url
    /// invariant).
    pub fn with_data(
        id: impl Into<String>,
        format: impl Into<String>,
        data: Struct,
    ) -> Self {
        Self {
            envelope3d_id: id.into(),
            format: format.into(),
            data: Some(data),
            url: String::new(),
            description: String::new(),
        }
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }
}

impl MobileRobotGeometry {
    pub fn new() -> Self {
        Self {
            wheel_definitions: Vec::new(),
            envelopes2d: Vec::new(),
            envelopes3d: Vec::new(),
        }
    }

    pub fn add_wheel(mut self, w: WheelDefinition) -> Self {
        self.wheel_definitions.push(w);
        self
    }

    pub fn add_envelope_2d(mut self, e: Envelope2D) -> Self {
        self.envelopes2d.push(e);
        self
    }

    pub fn add_envelope_3d(mut self, e: Envelope3D) -> Self {
        self.envelopes3d.push(e);
        self
    }
}

// ---------------------------------------------------------------------------
// LoadSpecification
// ---------------------------------------------------------------------------

impl LoadSet {
    /// Build a load set with the required identity and dimensions.
    pub fn new(
        set_name: impl Into<String>,
        load_type: impl Into<String>,
        bounding_box_reference: BoundingBoxReference,
        load_dimensions: BoundingBox,
    ) -> Self {
        Self {
            set_name: set_name.into(),
            load_type: load_type.into(),
            load_positions: Vec::new(),
            bounding_box_reference: Some(bounding_box_reference),
            load_dimensions: Some(load_dimensions),
            maximum_weight: None,
            minimum_load_handling_height: None,
            maximum_load_handling_height: None,
            minimum_load_handling_depth: None,
            maximum_load_handling_depth: None,
            minimum_load_handling_tilt: None,
            maximum_load_handling_tilt: None,
            maximum_speed: None,
            maximum_acceleration: None,
            maximum_deceleration: None,
            pick_time: None,
            drop_time: None,
            description: String::new(),
        }
    }

    pub fn with_height_envelope(mut self, min_m: f64, max_m: f64) -> Self {
        self.minimum_load_handling_height = Some(min_m);
        self.maximum_load_handling_height = Some(max_m);
        self
    }

    pub fn with_depth_envelope(mut self, min_m: f64, max_m: f64) -> Self {
        self.minimum_load_handling_depth = Some(min_m);
        self.maximum_load_handling_depth = Some(max_m);
        self
    }

    pub fn with_tilt_envelope(mut self, min_rad: f64, max_rad: f64) -> Self {
        self.minimum_load_handling_tilt = Some(min_rad);
        self.maximum_load_handling_tilt = Some(max_rad);
        self
    }

    pub fn with_motion_caps(mut self, max_speed: f64, max_accel: f64, max_decel: f64) -> Self {
        self.maximum_speed = Some(max_speed);
        self.maximum_acceleration = Some(max_accel);
        self.maximum_deceleration = Some(max_decel);
        self
    }

    pub fn with_maximum_weight(mut self, kg: f64) -> Self {
        self.maximum_weight = Some(kg);
        self
    }

    pub fn with_handling_times(mut self, pick_s: f64, drop_s: f64) -> Self {
        self.pick_time = Some(pick_s);
        self.drop_time = Some(drop_s);
        self
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    pub fn add_load_position(mut self, p: impl Into<String>) -> Self {
        self.load_positions.push(p.into());
        self
    }
}

impl LoadSpecification {
    pub fn new() -> Self {
        Self {
            load_positions: Vec::new(),
            load_sets: Vec::new(),
        }
    }

    pub fn add_load_position(mut self, p: impl Into<String>) -> Self {
        self.load_positions.push(p.into());
        self
    }

    pub fn add_load_set(mut self, s: LoadSet) -> Self {
        self.load_sets.push(s);
        self
    }
}

// ---------------------------------------------------------------------------
// MobileRobotConfiguration
// ---------------------------------------------------------------------------

impl Version {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl Network {
    pub fn new() -> Self {
        Self {
            dns_servers: Vec::new(),
            ntp_servers: Vec::new(),
            local_ip_address: String::new(),
            netmask: String::new(),
            default_gateway: String::new(),
        }
    }

    pub fn add_dns_server(mut self, s: impl Into<String>) -> Self {
        self.dns_servers.push(s.into());
        self
    }

    pub fn add_ntp_server(mut self, s: impl Into<String>) -> Self {
        self.ntp_servers.push(s.into());
        self
    }

    /// Combined setter for the three address fields that always travel
    /// together.
    pub fn with_static_address(
        mut self,
        ip: impl Into<String>,
        netmask: impl Into<String>,
        gateway: impl Into<String>,
    ) -> Self {
        self.local_ip_address = ip.into();
        self.netmask = netmask.into();
        self.default_gateway = gateway.into();
        self
    }
}

impl BatteryCharging {
    pub fn new() -> Self {
        Self {
            critical_low_charging_level: None,
            minimum_desired_charging_level: None,
            maximum_desired_charging_level: None,
            minimum_charging_time: None,
        }
    }

    /// Validate that `minimum ≤ maximum` on the charging band.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let (Some(min), Some(max)) = (
            self.minimum_desired_charging_level,
            self.maximum_desired_charging_level,
        ) {
            if min > max {
                return Err(ValidationError::OutOfRange {
                    field: "battery_charging.min_max_band",
                    value: format!("min={min} max={max}"),
                });
            }
        }
        Ok(())
    }
}

impl MobileRobotConfiguration {
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
            network: Some(Network::new()),
            battery_charging: Some(BatteryCharging::new()),
        }
    }

    pub fn add_version(mut self, v: Version) -> Self {
        self.versions.push(v);
        self
    }

    pub fn with_network(mut self, n: Network) -> Self {
        self.network = Some(n);
        self
    }

    pub fn with_battery_charging(mut self, b: BatteryCharging) -> Self {
        self.battery_charging = Some(b);
        self
    }
}
