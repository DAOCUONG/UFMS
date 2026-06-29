//! [Order], [Node], [Edge], [Corridor] constructors + builders.
//!
//! The spec note in `order.proto` reads:
//! > *"One node is enough for a valid order (leave edges empty in that case)."*
//!
//! [`OrderExt::single_node`] encodes that invariant directly.

use crate::error::ValidationError;
use crate::vda5050::v3::{
    Action, Corridor, CorridorReferencePoint, Edge, Header, Node, NodePosition, Order, OrientationType,
    ReleaseLossBehavior, Trajectory,
};

// ---------------------------------------------------------------------------
// Order constructors + semantic helpers
// ---------------------------------------------------------------------------

/// Ergonomic helpers for the [`Order`] message.
pub trait OrderExt {
    /// True when the order has at least one node (the minimal valid order
    /// per spec).
    fn is_valid_minimal(&self) -> bool;

    /// Count of distinct sequence IDs across nodes and edges (informational).
    fn distinct_sequence_ids(&self) -> usize;

    /// `Validate()` returns errors when both nodes and edges are empty.
    fn validate(&self) -> Result<(), ValidationError>;
}

impl OrderExt for Order {
    fn is_valid_minimal(&self) -> bool {
        !self.nodes.is_empty() || !self.edges.is_empty()
    }

    fn distinct_sequence_ids(&self) -> usize {
        let mut ids: Vec<u32> = self.nodes.iter().map(|n| n.sequence_id).collect();
        ids.extend(self.edges.iter().map(|e| e.sequence_id));
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.nodes.is_empty() && self.edges.is_empty() {
            return Err(ValidationError::EmptyOrder);
        }
        Ok(())
    }
}

impl Order {
    /// Build a new order with the given `order_id`. `order_update_id` starts
    /// at 0; `order_description` is empty; `nodes` and `edges` are empty.
    pub fn new(order_id: impl Into<String>) -> Self {
        Self {
            header: None,
            order_id: order_id.into(),
            order_update_id: 0,
            order_description: String::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Build a minimal valid order consisting of one node (no edges).
    pub fn single_node(order_id: impl Into<String>, node: Node) -> Self {
        Self {
            header: None,
            order_id: order_id.into(),
            order_update_id: 0,
            order_description: String::new(),
            nodes: vec![node],
            edges: Vec::new(),
        }
    }

    /// Chainable setter for `header`.
    pub fn with_header(mut self, header: Header) -> Self {
        self.header = Some(header);
        self
    }

    /// Chainable setter for `order_update_id`.
    pub fn with_update_id(mut self, n: u32) -> Self {
        self.order_update_id = n;
        self
    }

    /// Chainable setter for `order_description` (display only per spec).
    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.order_description = d.into();
        self
    }

    /// Chainable setter for `nodes`.
    pub fn with_nodes(mut self, nodes: Vec<Node>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Chainable setter for `edges`.
    pub fn with_edges(mut self, edges: Vec<Edge>) -> Self {
        self.edges = edges;
        self
    }

    /// Adds one node at the end.
    pub fn add_node(mut self, node: Node) -> Self {
        self.nodes.push(node);
        self
    }

    /// Adds one edge at the end.
    pub fn add_edge(mut self, edge: Edge) -> Self {
        self.edges.push(edge);
        self
    }
}

// ---------------------------------------------------------------------------
// Node constructors + builder
// ---------------------------------------------------------------------------

impl Node {
    /// Build a node with the given `node_id` and `sequence_id`. `released`
    /// defaults to `true` (node is part of the base).
    pub fn new(node_id: impl Into<String>, sequence_id: u32) -> Self {
        Self {
            node_id: node_id.into(),
            sequence_id,
            node_descriptor: String::new(),
            released: true,
            node_position: None,
            actions: Vec::new(),
        }
    }

    /// Chainable setter for `released`. `true` = base, `false` = horizon.
    pub fn released(mut self, released: bool) -> Self {
        self.released = released;
        self
    }

    /// Chainable setter for `node_descriptor`.
    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.node_descriptor = d.into();
        self
    }

    /// Chainable setter for `node_position`.
    pub fn with_position(mut self, p: NodePosition) -> Self {
        self.node_position = Some(p);
        self
    }

    /// Adds one action.
    pub fn add_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Replaces the action list.
    pub fn with_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    /// `true` when this node is part of the order's base (per spec).
    pub fn is_base(&self) -> bool {
        self.released
    }

    /// `true` when this node is part of the order's horizon (per spec).
    pub fn is_horizon(&self) -> bool {
        !self.released
    }
}

/// Fluent builder for [`Node`].
pub struct NodeBuilder(Node);

impl NodeBuilder {
    pub fn new(node_id: impl Into<String>, sequence_id: u32) -> Self {
        Self(Node::new(node_id, sequence_id))
    }

    pub fn released(mut self, r: bool) -> Self {
        self.0.released = r;
        self
    }

    pub fn descriptor(mut self, d: impl Into<String>) -> Self {
        self.0.node_descriptor = d.into();
        self
    }

    pub fn position(mut self, p: NodePosition) -> Self {
        self.0.node_position = Some(p);
        self
    }

    pub fn action(mut self, a: Action) -> Self {
        self.0.actions.push(a);
        self
    }

    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.0.actions = actions;
        self
    }

    pub fn build(self) -> Node {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Edge constructors + builder
// ---------------------------------------------------------------------------

impl Edge {
    /// Build an edge with the given `edge_id` and `sequence_id`. `released`
    /// defaults to `true`; `orientation_type` defaults to
    /// [`OrientationType::OrientationTangential`] per spec.
    pub fn new(edge_id: impl Into<String>, sequence_id: u32) -> Self {
        Self {
            edge_id: edge_id.into(),
            sequence_id,
            edge_descriptor: String::new(),
            released: true,
            maximum_speed: None,
            maximum_mobile_robot_height: None,
            minimum_load_handling_device_height: None,
            orientation: None,
            orientation_type: OrientationType::OrientationTangential as i32,
            direction: String::new(),
            reach_orientation_before_entering: None,
            max_rotation_speed: None,
            trajectory: None,
            length: None,
            corridor: None,
            actions: Vec::new(),
        }
    }

    /// Chainable setter for `released`.
    pub fn released(mut self, r: bool) -> Self {
        self.released = r;
        self
    }

    /// Chainable setter for `edge_descriptor`.
    pub fn with_descriptor(mut self, d: impl Into<String>) -> Self {
        self.edge_descriptor = d.into();
        self
    }

    /// Chainable setter for the optional physical-clearance fields that
    /// typically travel together.
    pub fn with_speed_limits(
        mut self,
        maximum_speed: f64,
        max_height: f64,
        min_load_handling_height: f64,
    ) -> Self {
        self.maximum_speed = Some(maximum_speed);
        self.maximum_mobile_robot_height = Some(max_height);
        self.minimum_load_handling_device_height = Some(min_load_handling_height);
        self
    }

    /// Chainable setter for the orientation cluster.
    pub fn with_orientation(
        mut self,
        orientation_rad: f64,
        orientation_type: OrientationType,
        reach_before_entering: bool,
        max_rotation_speed: Option<f64>,
    ) -> Self {
        self.orientation = Some(orientation_rad);
        self.orientation_type = orientation_type as i32;
        self.reach_orientation_before_entering = Some(reach_before_entering);
        self.max_rotation_speed = max_rotation_speed;
        self
    }

    /// Chainable setter for `direction` (line-guided set-direction).
    pub fn with_direction(mut self, d: impl Into<String>) -> Self {
        self.direction = d.into();
        self
    }

    /// Chainable setter for `trajectory`.
    pub fn with_trajectory(mut self, t: Trajectory) -> Self {
        self.trajectory = Some(t);
        self
    }

    /// Chainable setter for `length`.
    pub fn with_length(mut self, l: f64) -> Self {
        self.length = Some(l);
        self
    }

    /// Chainable setter for `corridor`.
    pub fn with_corridor(mut self, c: Corridor) -> Self {
        self.corridor = Some(c);
        self
    }

    /// Adds one action.
    pub fn add_action(mut self, a: Action) -> Self {
        self.actions.push(a);
        self
    }

    /// Replaces the action list.
    pub fn with_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }
}

/// Fluent builder for [`Edge`].
pub struct EdgeBuilder(Edge);

impl EdgeBuilder {
    pub fn new(edge_id: impl Into<String>, sequence_id: u32) -> Self {
        Self(Edge::new(edge_id, sequence_id))
    }

    pub fn released(mut self, r: bool) -> Self {
        self.0.released = r;
        self
    }

    pub fn descriptor(mut self, d: impl Into<String>) -> Self {
        self.0.edge_descriptor = d.into();
        self
    }

    pub fn maximum_speed(mut self, m: f64) -> Self {
        self.0.maximum_speed = Some(m);
        self
    }

    pub fn maximum_height(mut self, m: f64) -> Self {
        self.0.maximum_mobile_robot_height = Some(m);
        self
    }

    pub fn minimum_load_handling_height(mut self, m: f64) -> Self {
        self.0.minimum_load_handling_device_height = Some(m);
        self
    }

    pub fn orientation(mut self, theta: f64, kind: OrientationType) -> Self {
        self.0.orientation = Some(theta);
        self.0.orientation_type = kind as i32;
        self
    }

    pub fn reach_orientation_before_entering(mut self, r: bool) -> Self {
        self.0.reach_orientation_before_entering = Some(r);
        self
    }

    pub fn max_rotation_speed(mut self, w: f64) -> Self {
        self.0.max_rotation_speed = Some(w);
        self
    }

    pub fn direction(mut self, d: impl Into<String>) -> Self {
        self.0.direction = d.into();
        self
    }

    pub fn trajectory(mut self, t: Trajectory) -> Self {
        self.0.trajectory = Some(t);
        self
    }

    pub fn length(mut self, l: f64) -> Self {
        self.0.length = Some(l);
        self
    }

    pub fn corridor(mut self, c: Corridor) -> Self {
        self.0.corridor = Some(c);
        self
    }

    pub fn action(mut self, a: Action) -> Self {
        self.0.actions.push(a);
        self
    }

    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.0.actions = actions;
        self
    }

    pub fn build(self) -> Edge {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Corridor
// ---------------------------------------------------------------------------

/// Ergonomic helpers for [`Corridor`].
pub trait CorridorExt {
    /// Build a corridor with `left_width`, `right_width`, and a reference
    /// point. `release_required` defaults to `false`; `release_loss_behavior`
    /// defaults to [`ReleaseLossBehavior::ReleaseStop`] per spec.
    fn new(left_width: f64, right_width: f64, reference: CorridorReferencePoint) -> Self;
}

impl CorridorExt for Corridor {
    fn new(left_width: f64, right_width: f64, reference: CorridorReferencePoint) -> Self {
        Self {
            left_width,
            right_width,
            corridor_reference_point: reference as i32,
            release_required: None,
            release_loss_behavior: ReleaseLossBehavior::ReleaseStop as i32,
        }
    }
}

impl Corridor {
    /// Chainable setter for `release_required`.
    pub fn with_release_required(mut self, r: bool) -> Self {
        self.release_required = Some(r);
        self
    }

    /// Chainable setter for `release_loss_behavior`.
    pub fn with_release_loss_behavior(mut self, b: ReleaseLossBehavior) -> Self {
        self.release_loss_behavior = b as i32;
        self
    }
}

// ---------------------------------------------------------------------------
// OrderBuilder — chainable wrapper for the verbose Order message.
// ---------------------------------------------------------------------------

/// Fluent builder for [`Order`].
pub struct OrderBuilder(Order);

impl OrderBuilder {
    pub fn new(order_id: impl Into<String>) -> Self {
        Self(Order::new(order_id))
    }

    pub fn header(mut self, h: Header) -> Self {
        self.0.header = Some(h);
        self
    }

    pub fn update_id(mut self, n: u32) -> Self {
        self.0.order_update_id = n;
        self
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.0.order_description = d.into();
        self
    }

    pub fn node(mut self, n: Node) -> Self {
        self.0.nodes.push(n);
        self
    }

    pub fn nodes(mut self, nodes: Vec<Node>) -> Self {
        self.0.nodes = nodes;
        self
    }

    pub fn edge(mut self, e: Edge) -> Self {
        self.0.edges.push(e);
        self
    }

    pub fn edges(mut self, edges: Vec<Edge>) -> Self {
        self.0.edges = edges;
        self
    }

    /// Build the message. Returns [`ValidationError::EmptyOrder`] when no
    /// nodes and no edges are present.
    pub fn try_build(self) -> Result<Order, ValidationError> {
        self.0.validate()?;
        Ok(self.0)
    }

    pub fn build(self) -> Order {
        self.0
    }
}

impl From<OrderBuilder> for Order {
    fn from(b: OrderBuilder) -> Self {
        b.build()
    }
}
