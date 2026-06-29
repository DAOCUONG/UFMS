//! [Visualization] builder.
//!
//! Per `visualization.proto` (file-level comment): all payload fields except
//! `header` and `reference_state_header_id` are optional to save bandwidth.
//! [`VisualizationBuilder`] exposes them as chainable setters and provides a
//! convenience for the empty-body case.

use crate::vda5050::v3::{
    Header, IntermediatePath, MobileRobotPosition, PlannedPath, Velocity, Visualization,
};

impl Visualization {
    /// Build a `Visualization` referencing the given state-header-id. All
    /// payload fields default to `None`.
    pub fn new(header: Header, reference_state_header_id: u32) -> Self {
        Self {
            header: Some(header),
            reference_state_header_id,
            planned_path: None,
            intermediate_path: None,
            mobile_robot_position: None,
            velocity: None,
        }
    }

    /// Convenience: empty-body visualization tied to a state header id (saves
    /// bandwidth when nothing has changed).
    pub fn empty_for_state(header: Header, reference_state_header_id: u32) -> Self {
        Self::new(header, reference_state_header_id)
    }

    /// Chainable setter for `planned_path`.
    pub fn with_planned_path(mut self, p: PlannedPath) -> Self {
        self.planned_path = Some(p);
        self
    }

    /// Chainable setter for `intermediate_path`.
    pub fn with_intermediate_path(mut self, p: IntermediatePath) -> Self {
        self.intermediate_path = Some(p);
        self
    }

    /// Chainable setter for `mobile_robot_position`.
    pub fn with_position(mut self, p: MobileRobotPosition) -> Self {
        self.mobile_robot_position = Some(p);
        self
    }

    /// Chainable setter for `velocity`.
    pub fn with_velocity(mut self, v: Velocity) -> Self {
        self.velocity = Some(v);
        self
    }
}

/// Fluent builder for [`Visualization`].
pub struct VisualizationBuilder(Visualization);

impl VisualizationBuilder {
    pub fn new(header: Header, reference_state_header_id: u32) -> Self {
        Self(Visualization::new(header, reference_state_header_id))
    }

    pub fn planned_path(mut self, p: PlannedPath) -> Self {
        self.0.planned_path = Some(p);
        self
    }

    pub fn intermediate_path(mut self, p: IntermediatePath) -> Self {
        self.0.intermediate_path = Some(p);
        self
    }

    pub fn position(mut self, p: MobileRobotPosition) -> Self {
        self.0.mobile_robot_position = Some(p);
        self
    }

    pub fn velocity(mut self, v: Velocity) -> Self {
        self.0.velocity = Some(v);
        self
    }

    pub fn build(self) -> Visualization {
        self.0
    }
}
