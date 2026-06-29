//! Geometric helpers — [`Vertex2D`], [`NodePosition`], [`Trajectory`],
//! [`BoundingBox`], polygon area / perimeter / containment / centroid.
//!
//! Keeps the ergonomic layer self-contained: a rust consumer can build,
//! transform, and validate every geometric primitive without pulling in
//! `glam` / `nalgebra` / `geo`.

use std::ops::{Add, Sub};

use crate::error::ValidationError;
use crate::vda5050::v3::{
    BoundingBox, BoundingBoxReference, ControlPoint, NodePosition, NodePositionDeviation, Trajectory,
    Vertex2D,
};

// ---------------------------------------------------------------------------
// Vertex2D
// ---------------------------------------------------------------------------

impl Vertex2D {
    /// Build a 2D vertex.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another vertex, in meters.
    pub fn distance_to(&self, other: Vertex2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Translate this vertex by `(dx, dy)`.
    pub fn translated(self, dx: f64, dy: f64) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Rotate this vertex counter-clockwise by `theta_rad` radians around
    /// the origin.
    pub fn rotated(self, theta_rad: f64) -> Self {
        let (s, c) = theta_rad.sin_cos();
        Self {
            x: c * self.x - s * self.y,
            y: s * self.x + c * self.y,
        }
    }

    /// Rotate this vertex around a pivot by `theta_rad`.
    pub fn rotated_about(self, pivot: Vertex2D, theta_rad: f64) -> Self {
        let relative = Vertex2D {
            x: self.x - pivot.x,
            y: self.y - pivot.y,
        }
        .rotated(theta_rad);
        Vertex2D {
            x: relative.x + pivot.x,
            y: relative.y + pivot.y,
        }
    }

    /// Dot product (treating the vertex as a 2D vector).
    pub fn dot(&self, other: Vertex2D) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl From<(f64, f64)> for Vertex2D {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

impl From<Vertex2D> for (f64, f64) {
    fn from(v: Vertex2D) -> Self {
        (v.x, v.y)
    }
}

impl Add for Vertex2D {
    type Output = Vertex2D;
    fn add(self, rhs: Vertex2D) -> Vertex2D {
        Vertex2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vertex2D {
    type Output = Vertex2D;
    fn sub(self, rhs: Vertex2D) -> Vertex2D {
        Vertex2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

// ---------------------------------------------------------------------------
// Polygon helpers
// ---------------------------------------------------------------------------

/// Signed area of the polygon (positive = counter-clockwise). For
/// areas regardless of winding order, use [`polygon_area_unsigned`].
pub fn polygon_area(vertices: &[Vertex2D]) -> f64 {
    if vertices.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..vertices.len() {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];
        sum += (a.x * b.y) - (b.x * a.y);
    }
    sum / 2.0
}

/// Always-positive area.
pub fn polygon_area_unsigned(vertices: &[Vertex2D]) -> f64 {
    polygon_area(vertices).abs()
}

/// Perimeter — sum of edge lengths. The polygon is assumed closed (last →
/// first edge is included).
pub fn polygon_perimeter(vertices: &[Vertex2D]) -> f64 {
    if vertices.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..vertices.len() {
        sum += vertices[i].distance_to(vertices[(i + 1) % vertices.len()]);
    }
    sum
}

/// Geometric centroid. Returns the origin if the polygon is degenerate.
pub fn polygon_centroid(vertices: &[Vertex2D]) -> Vertex2D {
    if vertices.is_empty() {
        return Vertex2D { x: 0.0, y: 0.0 };
    }
    if vertices.len() < 3 {
        // Fall back to the simple average for degenerate inputs.
        let n = vertices.len() as f64;
        let sx = vertices.iter().map(|v| v.x).sum::<f64>();
        let sy = vertices.iter().map(|v| v.y).sum::<f64>();
        return Vertex2D { x: sx / n, y: sy / n };
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut a2 = 0.0;
    for i in 0..vertices.len() {
        let p1 = vertices[i];
        let p2 = vertices[(i + 1) % vertices.len()];
        let cross = p1.x * p2.y - p2.x * p1.y;
        cx += (p1.x + p2.x) * cross;
        cy += (p1.y + p2.y) * cross;
        a2 += cross;
    }
    if a2.abs() < f64::EPSILON {
        return polygon_centroid_degenerate(vertices);
    }
    Vertex2D { x: cx / (3.0 * a2), y: cy / (3.0 * a2) }
}

fn polygon_centroid_degenerate(vertices: &[Vertex2D]) -> Vertex2D {
    let n = vertices.len() as f64;
    let sx = vertices.iter().map(|v| v.x).sum::<f64>();
    let sy = vertices.iter().map(|v| v.y).sum::<f64>();
    Vertex2D { x: sx / n, y: sy / n }
}

/// Axis-aligned bounding box — `(min, max)` corners.
pub fn polygon_bounding_box(vertices: &[Vertex2D]) -> (Vertex2D, Vertex2D) {
    if vertices.is_empty() {
        return (Vertex2D::new(0.0, 0.0), Vertex2D::new(0.0, 0.0));
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for v in &vertices[1..] {
        if v.x < min.x {
            min.x = v.x;
        }
        if v.y < min.y {
            min.y = v.y;
        }
        if v.x > max.x {
            max.x = v.x;
        }
        if v.y > max.y {
            max.y = v.y;
        }
    }
    (min, max)
}

/// Ray-casting point-in-polygon test. The polygon is treated as the boundary
/// of a simple region; points on an edge are reported inside.
pub fn point_in_polygon(point: Vertex2D, vertices: &[Vertex2D]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let n = vertices.len();
    let mut j = n - 1;
    for i in 0..n {
        let pi = vertices[i];
        let pj = vertices[j];
        if (pi.y > point.y) != (pj.y > point.y) {
            let x_intersect = pj.x + (point.y - pj.y) * (pi.x - pj.x) / (pi.y - pj.y);
            if point.x < x_intersect {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

// ---------------------------------------------------------------------------
// NodePosition
// ---------------------------------------------------------------------------

impl NodePosition {
    /// Build a node position with required `(x, y, map_id)` only; `theta`
    /// and deviations default to unset.
    pub fn from_xy_theta(
        x: f64,
        y: f64,
        theta: Option<f64>,
        map_id: impl Into<String>,
    ) -> Self {
        Self {
            x,
            y,
            theta,
            allowed_deviation_xy: None,
            allowed_deviation_theta: None,
            map_id: map_id.into(),
        }
    }

    /// Chainable setter for the XY deviation ellipse.
    pub fn with_xy_deviation(mut self, dev: NodePositionDeviation) -> Self {
        self.allowed_deviation_xy = Some(dev);
        self
    }

    /// Chainable setter for the theta deviation.
    pub fn with_theta_deviation(mut self, dev: f64) -> Self {
        self.allowed_deviation_theta = Some(dev);
        self
    }
}

impl NodePositionDeviation {
    /// Build a deviation ellipse with semi-axes `a` and `b` and rotation
    /// `theta_rad`.
    pub fn new(a: f64, b: f64, theta_rad: f64) -> Self {
        Self {
            a,
            b,
            theta: theta_rad,
        }
    }
}

impl ControlPoint {
    /// Build a control point with the default weight of 1.0.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, weight: None }
    }

    /// Chainable setter for the control-point weight.
    pub fn with_weight(mut self, w: f64) -> Self {
        self.weight = Some(w);
        self
    }
}

// ---------------------------------------------------------------------------
// Trajectory
// ---------------------------------------------------------------------------

impl Trajectory {
    /// The number of knot values a valid NURBS trajectory must have, given
    /// `n` control points and degree `d`: `n + d + 1`.
    pub fn expected_knot_count(n_control_points: usize, degree: u32) -> usize {
        n_control_points + degree as usize + 1
    }

    /// Build a minimal linear NURBS (degree 1) through the given control
    /// points. For linear interpolation, `knot_vector.len() ==
    /// control_points.len() + 2`.
    pub fn linear(control_points: Vec<ControlPoint>) -> Self {
        let n = control_points.len();
        let mut knot_vector: Vec<f64> = (0..=n + 1).map(|i| i as f64 / (n + 1) as f64).collect();
        if knot_vector.is_empty() {
            knot_vector = vec![0.0, 1.0];
        }
        Self {
            degree: Some(1),
            knot_vector,
            control_points,
        }
    }

    /// Validate that the knot vector length is correct for the given
    /// `control_points` and `degree`. Per VDA 5050 spec:
    /// `knot_vector.len() == control_points.len() + degree + 1`.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let degree = self.degree.unwrap_or(1) as usize;
        let expected = Self::expected_knot_count(self.control_points.len(), degree as u32);
        let got = self.knot_vector.len();
        if got != expected {
            return Err(ValidationError::KnotVectorMismatch {
                got,
                expected,
                control_points: self.control_points.len(),
                degree: degree as u32,
            });
        }
        if self.knot_vector.iter().any(|&k| !(0.0..=1.0).contains(&k)) {
            return Err(ValidationError::OutOfRange {
                field: "knot_vector",
                value: "outside [0,1]".into(),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// BoundingBox / BoundingBoxReference
// ---------------------------------------------------------------------------

impl BoundingBox {
    /// Build a bounding box with required `length` and `width`; height
    /// defaults to unset.
    pub fn new(length: f64, width: f64) -> Self {
        Self {
            length,
            width,
            height: None,
        }
    }

    /// Chainable setter for `height`.
    pub fn with_height(mut self, h: f64) -> Self {
        self.height = Some(h);
        self
    }

    /// Floor-area (length × width), regardless of whether height is set.
    pub fn floor_area(&self) -> f64 {
        self.length * self.width
    }

    /// Volume (length × width × height). Returns `None` when height is unset.
    pub fn volume(&self) -> Option<f64> {
        self.height.map(|h| self.length * self.width * h)
    }
}

impl BoundingBoxReference {
    /// Build a bounding-box reference point centered at the bottom surface
    /// of the load's bounding box.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            theta: None,
        }
    }

    /// Chainable setter for the bounding-box rotation (for tugger trains).
    pub fn with_theta(mut self, theta_rad: f64) -> Self {
        self.theta = Some(theta_rad);
        self
    }
}
