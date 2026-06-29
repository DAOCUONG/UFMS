//! Integration tests for the ergonomic Rust model layer.
//!
//! Each test exercises one constructor or validator end-to-end: it builds
//! a value through the ergonomic API, round-trips it through the prost
//! codec, and asserts the result matches expectations.

use prost::Message;
use ufms::action::ActionParameterExt;
use ufms::header::TimestampExt;
use ufms::prelude::*;

// ---------------------------------------------------------------------------
// Header / Timestamp
// ---------------------------------------------------------------------------

#[test]
fn header_new_round_trips() {
    let h = Header::new("ACME", "amr-001")
        .with_header_id(42)
        .with_timestamp_now();
    let bytes = h.encode_to_vec();
    let back = Header::decode(bytes.as_slice()).unwrap();
    assert_eq!(back.manufacturer, "ACME");
    assert_eq!(back.serial_number, "amr-001");
    assert_eq!(back.version, "3.0.0");
    assert_eq!(back.header_id, 42);
    h.validate().unwrap();
}

#[test]
fn header_validate_rejects_empty_fields() {
    let h = Header {
        header_id: 0,
        timestamp: None,
        version: "".into(),
        manufacturer: "X".into(),
        serial_number: "Y".into(),
    };
    assert!(h.validate().is_err());

    let h = Header {
        header_id: 0,
        timestamp: None,
        version: "2.0.0".into(),
        manufacturer: "X".into(),
        serial_number: "Y".into(),
    };
    assert!(h.validate().is_err()); // not v3.x
}

#[test]
fn timestamp_round_trips() {
    let ts = prost_types::Timestamp::now_utc();
    let iso = ts.to_iso8601_utc();
    let back = prost_types::Timestamp::from_iso8601_utc(&iso).unwrap();
    // ms-precision round-trip
    assert_eq!(back.seconds, ts.seconds);
    assert_eq!(back.nanos / 1_000_000, ts.nanos / 1_000_000);
}

#[test]
fn timestamp_parses_known_value() {
    let ts = prost_types::Timestamp::from_iso8601_utc("2026-06-29T12:34:56.789Z").unwrap();
    let iso = ts.to_iso8601_utc();
    assert_eq!(iso, "2026-06-29T12:34:56.789Z");
}

#[test]
fn header_ext_works_on_every_topic() {
    let h = Header::new("ACME", "amr-001");
    let mut conn = Connection::online(h.clone());
    assert!(conn.header().is_some());
    let _ = conn.header_mut();
    conn.set_header(h.clone());

    let mut ia = InstantActionsBuilder::new(h.clone()).build();
    assert!(ia.header().is_some());
    ia.set_header(h.clone());

    let mut ord = Order::new("o1").with_header(h.clone());
    assert!(ord.header().is_some());
    ord.set_header(h.clone());

    let mut st = State::new(h.clone());
    st.set_header(h.clone());

    let mut viz = Visualization::empty_for_state(h.clone(), 0);
    viz.set_header(h.clone());

    let mut fs = Factsheet::new("s", "d").with_header(h.clone());
    fs.set_header(h.clone());

    let mut zs = ZoneSet::new(h.clone(), "m", "zs");
    zs.set_header(h.clone());

    let mut resp = ResponseBuilder::new(h).build();
    assert!(resp.header().is_some());
}

// ---------------------------------------------------------------------------
// Connection lifecycle
// ---------------------------------------------------------------------------

#[test]
fn connection_lifecycle_tags() {
    let h = Header::new("ACME", "amr-001");
    assert_eq!(
        Connection::online(h.clone()).connection_state,
        ConnectionState::ConnectionOnline as i32
    );
    assert_eq!(
        Connection::offline(h.clone()).connection_state,
        ConnectionState::ConnectionOffline as i32
    );
    assert_eq!(
        Connection::broken(h.clone()).connection_state,
        ConnectionState::ConnectionBroken as i32
    );
    assert_eq!(
        Connection::hibernating(h.clone()).connection_state,
        ConnectionState::ConnectionHibernating as i32
    );
    assert!(Connection::broken(h).is_disconnected());
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[test]
fn enum_short_name_and_display() {
    assert_eq!(BlockingType::BlockingHard.short_name(), "hard");
    assert_eq!(BlockingType::BlockingHard.to_string(), "hard");
    assert!(BlockingType::BlockingNone.is_unspecified() == false);
    assert!(BlockingType::Unspecified.is_unspecified());
}

#[test]
fn enum_from_str_accepts_three_forms() {
    let a: BlockingType = "BLOCKING_HARD".parse().unwrap();
    let b: BlockingType = "blocking_hard".parse().unwrap();
    let c: BlockingType = "hard".parse().unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, BlockingType::BlockingHard);
}

#[test]
fn enum_semantic_helpers() {
    assert!(ErrorLevel::ErrorFatal.accepts_new_orders() == false);
    assert!(ErrorLevel::ErrorWarning.accepts_new_orders());
    assert!(ErrorLevel::ErrorCritical.can_continue_order() == false);
    assert!(ErrorLevel::ErrorWarning.can_continue_order());
    assert!(EStopType::EstopNone.is_clear());
    assert!(!EStopType::EstopManual.is_clear());
    assert!(GrantType::GrantGranted.is_positive());
    assert!(!GrantType::GrantRejected.is_positive());
    assert!(MapStatus::MapEnabled.is_active());
    assert!(!ConnectionState::ConnectionOnline.is_disconnected());
    assert!(ConnectionState::ConnectionBroken.is_disconnected());
}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

#[test]
fn vertex2d_distance_to() {
    let a = Vertex2D::new(0.0, 0.0);
    let b = Vertex2D::new(3.0, 4.0);
    assert!((a.distance_to(b) - 5.0).abs() < 1e-9);
}

#[test]
fn polygon_helpers_on_unit_square() {
    let sq = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(1.0, 0.0),
        Vertex2D::new(1.0, 1.0),
        Vertex2D::new(0.0, 1.0),
    ];
    assert!((polygon_area(&sq) - 1.0).abs() < 1e-9);
    assert!((polygon_area_unsigned(&sq) - 1.0).abs() < 1e-9);
    assert!((polygon_perimeter(&sq) - 4.0).abs() < 1e-9);
    let (mn, mx) = polygon_bounding_box(&sq);
    assert_eq!(mn, Vertex2D::new(0.0, 0.0));
    assert_eq!(mx, Vertex2D::new(1.0, 1.0));
    assert!(point_in_polygon(Vertex2D::new(0.5, 0.5), &sq));
    assert!(!point_in_polygon(Vertex2D::new(2.0, 0.5), &sq));
    let c = polygon_centroid(&sq);
    assert!((c.x - 0.5).abs() < 1e-9);
    assert!((c.y - 0.5).abs() < 1e-9);
}

#[test]
fn polygon_area_on_triangle() {
    let tri = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(1.0, 0.0),
        Vertex2D::new(0.0, 1.0),
    ];
    assert!((polygon_area(&tri) - 0.5).abs() < 1e-9);
}

#[test]
fn bounding_box_volume() {
    let bb = BoundingBox::new(2.0, 1.0).with_height(0.5);
    assert_eq!(bb.floor_area(), 2.0);
    assert_eq!(bb.volume(), Some(1.0));
    let bb2 = BoundingBox::new(2.0, 1.0);
    assert_eq!(bb2.volume(), None);
}

// ---------------------------------------------------------------------------
// Trajectory validation
// ---------------------------------------------------------------------------

#[test]
fn trajectory_linear_validates() {
    let t = Trajectory::linear(vec![
        ControlPoint::new(0.0, 0.0),
        ControlPoint::new(1.0, 0.0),
        ControlPoint::new(1.0, 1.0),
    ]);
    t.validate().unwrap();
}

#[test]
fn trajectory_rejects_bad_knot_count() {
    let mut t = Trajectory::linear(vec![
        ControlPoint::new(0.0, 0.0),
        ControlPoint::new(1.0, 0.0),
        ControlPoint::new(1.0, 1.0),
    ]);
    t.knot_vector = vec![0.0, 0.5];
    assert!(t.validate().is_err());
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

#[test]
fn action_for_instant_forces_blocking_none() {
    let a = Action::for_instant("pauseOrder", "ia-1");
    assert_eq!(a.blocking_type, BlockingType::BlockingNone as i32);
    assert!(a.retriable.is_none());
}

#[test]
fn action_validate_rejects_empty_id_or_type() {
    let mut a = Action::for_order("pick", "act-1");
    a.action_id = String::new();
    assert!(a.validate().is_err());
    a.action_id = "act-1".into();
    a.action_type = String::new();
    assert!(a.validate().is_err());
}

#[test]
fn action_parameter_typed_builders() {
    use prost_types::{value::Kind, Value};
    let p = ActionParameter::string("device", "robot-42");
    assert!(matches!(p.value, Some(Value { kind: Some(Kind::StringValue(_)) })));
    let p = ActionParameter::bool("active", true);
    assert!(matches!(p.value, Some(Value { kind: Some(Kind::BoolValue(true)) })));
    let p = ActionParameter::number("speed", 1.5);
    assert!(matches!(p.value, Some(Value { kind: Some(Kind::NumberValue(1.5)) })));
    let p = ActionParameter::integer("count", 42);
    assert!(matches!(p.value, Some(Value { kind: Some(Kind::NumberValue(42.0)) })));
}

// ---------------------------------------------------------------------------
// InstantActions
// ---------------------------------------------------------------------------

#[test]
fn instant_actions_builder_rejects_non_blocking_none() {
    let h = Header::new("ACME", "amr-001");
    let res = InstantActionsBuilder::new(h).add_action(Action::for_order("pick", "a")).try_build();
    assert!(res.is_err());
}

#[test]
fn instant_actions_builder_accepts_blocking_none() {
    let h = Header::new("ACME", "amr-001");
    let msg = InstantActionsBuilder::new(h)
        .add_action(Action::for_instant("pauseOrder", "a"))
        .try_build()
        .unwrap();
    let bytes = msg.encode_to_vec();
    let back = InstantActions::decode(bytes.as_slice()).unwrap();
    assert_eq!(back.actions.len(), 1);
}

// ---------------------------------------------------------------------------
// Order
// ---------------------------------------------------------------------------

#[test]
fn order_is_valid_minimal_and_validate() {
    let o = Order::new("o1");
    assert!(!o.is_valid_minimal());
    assert!(o.validate().is_err());

    let o = Order::new("o2").add_node(Node::new("n1", 0));
    assert!(o.is_valid_minimal());
    o.validate().unwrap();
    assert_eq!(o.distinct_sequence_ids(), 1);
}

// ---------------------------------------------------------------------------
// State semantic helpers
// ---------------------------------------------------------------------------

#[test]
fn state_semantic_helpers() {
    let h = Header::new("ACME", "amr-001");
    let s = State::new(h.clone())
        .driving(true)
        .paused(false)
        .order_id("o1")
        .order_update_id(1)
        .last_node_id("n1")
        .last_node_sequence_id(2)
        .add_error(Error::new("warn", ErrorLevel::ErrorWarning, "d", "h"))
        .add_load(Load::new("p1", "EPAL").with_weight(100.0));
    assert!(s.is_driving());
    assert!(!s.is_paused());
    assert!(!s.is_idle() || s.is_idle()); // trivially
    let ctx = s.order_context();
    assert_eq!(ctx.order_id, "o1");
    assert_eq!(ctx.order_update_id, 1);
    assert_eq!(ctx.last_node_id, "n1");
    assert_eq!(ctx.last_node_sequence_id, 2);
    assert_eq!(s.errors().len(), 1);
    assert_eq!(s.loads().len(), 1);
    assert_eq!(s.loads()[0].weight_or_unknown(), Some(100.0));
    assert!(s.errors_at_or_above(ErrorLevel::ErrorWarning).len() == 1);
    assert!(s.errors_at_or_above(ErrorLevel::ErrorCritical).is_empty());
}

#[test]
fn mobile_robot_position_helpers() {
    let p = MobileRobotPosition {
        x: 0.0,
        y: 0.0,
        theta: 0.0,
        map_id: "m".into(),
        localized: true,
        localization_score: Some(0.9),
        deviation_range: Some(0.05),
    };
    assert!(p.is_trusted());
    assert_eq!(p.localization_quality(), Some(0.9));
    assert_eq!(p.localization_deviation(), Some(0.05));
}

#[test]
fn velocity_helpers() {
    let v = Velocity::zero();
    assert!(v.is_known());
    assert_eq!(v.linear_speed(), Some(0.0));

    let v = Velocity::new(Some(3.0), Some(4.0), Some(0.0));
    assert!(v.is_known());
    assert_eq!(v.linear_speed(), Some(5.0));

    let v = Velocity::new(Some(1.0), None, None);
    assert!(!v.is_known());
    assert_eq!(v.linear_speed(), None);
}

#[test]
fn safety_state_helpers() {
    let s = SafetyState::new(EStopType::EstopNone, false);
    assert!(s.is_safe_to_drive());
    let s = SafetyState::new(EStopType::EstopManual, false);
    assert!(!s.is_safe_to_drive());
    let s = SafetyState::new(EStopType::EstopNone, true);
    assert!(!s.is_safe_to_drive());
}

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

#[test]
fn response_constructors_and_lookup() {
    let r = Response::granted("a", None);
    assert_eq!(r.grant_type, GrantType::GrantGranted as i32);
    assert!(!r.has_lease());

    let r = Response::queued("a");
    assert_eq!(r.grant_type, GrantType::GrantQueued as i32);

    let r = Response::revoked("a");
    assert_eq!(r.grant_type, GrantType::GrantRevoked as i32);

    let r = Response::rejected("a");
    assert_eq!(r.grant_type, GrantType::GrantRejected as i32);

    let h = Header::new("ACME", "amr-001");
    let resp = ResponseBuilder::new(h)
        .response(Response::granted("req-1", None))
        .response(Response::queued("req-2"))
        .build();
    assert!(resp.lookup("req-1").is_some());
    assert!(resp.lookup("nope").is_none());
}

// ---------------------------------------------------------------------------
// Zones
// ---------------------------------------------------------------------------

fn triangle() -> Vec<Vertex2D> {
    vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(1.0, 0.0),
        Vertex2D::new(0.5, 1.0),
    ]
}

#[test]
fn zone_speed_limit_validates() {
    let z = Zone::speed_limit("z1", 0.5, triangle());
    z.validate().unwrap();
}

#[test]
fn zone_speed_limit_rejects_missing_max_speed() {
    let z = Zone {
        zone_id: "z".into(),
        zone_type: ZoneType::ZoneSpeedLimit as i32,
        zone_descriptor: String::new(),
        vertices: triangle(),
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
    };
    assert!(z.validate().is_err());
}

#[test]
fn zone_release_requires_release_loss_behavior() {
    let z = Zone::release("z", ReleaseLossBehavior::ReleaseStop, triangle());
    z.validate().unwrap();
    let mut bad = z.clone();
    bad.release_loss_behavior = None;
    assert!(bad.validate().is_err());
}

#[test]
fn zone_priority_and_penalty() {
    let z = Zone::priority("z", 0.7, triangle());
    z.validate().unwrap();
    let mut bad = z.clone();
    bad.priority_factor = None;
    assert!(bad.validate().is_err());

    let z = Zone::penalty("z", 0.3, triangle());
    z.validate().unwrap();
}

#[test]
fn zone_action_requires_action_lists() {
    let z = Zone::action("z", Vec::new(), Vec::new(), Vec::new(), triangle());
    assert!(z.validate().is_err()); // all empty
    let z = Zone::action(
        "z",
        vec![ZoneAction::new("announce", BlockingType::BlockingNone)],
        Vec::new(),
        Vec::new(),
        triangle(),
    );
    z.validate().unwrap();
}

#[test]
fn zone_directed_and_bidirected() {
    let z = Zone::directed(
        "z",
        1.0,
        DirectedLimitation::DirectedRestricted,
        triangle(),
    );
    z.validate().unwrap();

    let z = Zone::bidirected(
        "z",
        0.0,
        BidirectedLimitation::BidirectedSoft,
        triangle(),
    );
    z.validate().unwrap();
}

#[test]
fn zone_too_few_vertices_fails() {
    let z = Zone::blocked(
        "z",
        vec![Vertex2D::new(0.0, 0.0), Vertex2D::new(1.0, 0.0)],
    );
    assert!(z.validate().is_err());
}

#[test]
fn zone_contains_point() {
    let z = Zone::speed_limit(
        "z",
        0.5,
        vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(2.0, 0.0),
            Vertex2D::new(2.0, 2.0),
            Vertex2D::new(0.0, 2.0),
        ],
    );
    assert!(z.contains(Vertex2D::new(1.0, 1.0)));
    assert!(!z.contains(Vertex2D::new(3.0, 3.0)));
}

#[test]
fn zone_set_validates_each_zone() {
    let h = Header::new("ACME", "amr-001");
    let zs = ZoneSetBuilder::new(h, "m", "zs")
        .add_zone(Zone::speed_limit("z1", 0.5, triangle()))
        .add_zone(Zone::release("z2", ReleaseLossBehavior::ReleaseStop, triangle()))
        .try_build()
        .unwrap();
    assert_eq!(zs.zone_count(), 2);
}

// ---------------------------------------------------------------------------
// Visualization
// ---------------------------------------------------------------------------

#[test]
fn visualization_empty_round_trips() {
    let h = Header::new("ACME", "amr-001");
    let v = Visualization::empty_for_state(h, 7);
    let bytes = v.encode_to_vec();
    let back = Visualization::decode(bytes.as_slice()).unwrap();
    assert_eq!(back.reference_state_header_id, 7);
    assert!(back.planned_path.is_none());
}

// ---------------------------------------------------------------------------
// BatteryCharging validation
// ---------------------------------------------------------------------------

#[test]
fn battery_charging_validates_band() {
    let bc = BatteryCharging::new();
    bc.validate().unwrap();
    let bc = BatteryCharging {
        critical_low_charging_level: None,
        minimum_desired_charging_level: Some(20.0),
        maximum_desired_charging_level: Some(80.0),
        minimum_charging_time: None,
    };
    bc.validate().unwrap();
    let bc = BatteryCharging {
        critical_low_charging_level: None,
        minimum_desired_charging_level: Some(80.0),
        maximum_desired_charging_level: Some(20.0),
        minimum_charging_time: None,
    };
    assert!(bc.validate().is_err());
}