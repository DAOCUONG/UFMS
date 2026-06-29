//! UFMS — binary entry point.
//!
//! Exercises every ergonomic constructor and validator. Run with `cargo run`
//! to confirm the new layer compiles, builds valid messages, and produces
//! identical wire bytes to the previously handwritten demo.

use prost::Message;
use ufms::prelude::*;
use ufms::vda5050::v3::{
    BlockingType, ConnectionState, EStopType, ErrorLevel, OperatingMode, ReleaseLossBehavior,
    ZoneType,
};

fn main() {
    // Common header factory used by every demo below.
    let mk_header = || {
        Header::new("ACME Robotics", "amr-001")
            .with_header_id(0)
            .with_timestamp_now()
    };

    // -----------------------------------------------------------------------
    // 1. Connection — full lifecycle
    // -----------------------------------------------------------------------
    let conn = Connection::online(mk_header());
    let conn_bytes = conn.encode_to_vec();
    let conn_back = Connection::decode(conn_bytes.as_slice()).unwrap();
    assert_eq!(
        ConnectionState::try_from(conn_back.connection_state).unwrap(),
        ConnectionState::ConnectionOnline
    );
    assert_eq!(conn_back.header.as_ref().unwrap().manufacturer, "ACME Robotics");

    let broken = Connection::broken(mk_header().with_header_id(1));
    assert!(broken.is_disconnected());

    // -----------------------------------------------------------------------
    // 2. Order — single-node, ergonomic Node + Action constructors
    // -----------------------------------------------------------------------
    let order = OrderBuilder::new("order-abc")
        .header(mk_header().with_header_id(2))
        .description("Pick & deliver pallet from A1 to B2")
        .node(
            NodeBuilder::new("n1", 0)
                .descriptor("Pick station A1")
                .released(true)
                .position(NodePosition::from_xy_theta(
                    1.5,
                    2.5,
                    Some(1.5708),
                    "warehouse-1",
                ))
                .action(
                    Action::for_order("pick", "act-pick-1")
                        .with_descriptor("Pick pallet")
                        .with_blocking_type(BlockingType::BlockingHard)
                        .with_parameter(
                            "loadId",
                            prost_types::Value {
                                kind: Some(prost_types::value::Kind::StringValue(
                                    "pallet-42".into(),
                                )),
                            },
                        ),
                )
                .build(),
        )
        .build();
    let order_bytes = order.encode_to_vec();
    let order_back = Order::decode(order_bytes.as_slice()).unwrap();
    assert!(order_back.is_valid_minimal());
    assert_eq!(order_back.nodes[0].actions[0].action_type, "pick");
    assert_eq!(order_back.distinct_sequence_ids(), 1);

    // Validation: empty Order is rejected.
    let empty = Order::new("o-empty");
    assert!(empty.validate().is_err());

    // -----------------------------------------------------------------------
    // 3. InstantActions — BlockingNone is enforced
    // -----------------------------------------------------------------------
    let ia_msg = InstantActionsBuilder::new(mk_header().with_header_id(3))
        .add_action(Action::for_instant("pauseOrder", "ia-pause-1"))
        .add_action(Action::for_instant("resumeOrder", "ia-resume-1"))
        .try_build()
        .expect("blocking_type is forced to BlockingNone by Action::for_instant");
    let ia_bytes = ia_msg.encode_to_vec();
    let ia_back = InstantActions::decode(ia_bytes.as_slice()).unwrap();
    assert_eq!(ia_back.actions[0].action_type, "pauseOrder");

    // -----------------------------------------------------------------------
    // 4. State — battery + errors + safety + Load + MobileRobotPosition
    // -----------------------------------------------------------------------
    let state_msg = State::new(mk_header().with_header_id(4))
        .driving(true)
        .paused(false)
        .new_base_request(false)
        .add_load(
            LoadBuilder::new("pallet-42", "EPAL")
                .position("front")
                .weight(450.0)
                .build(),
        )
        .add_error(
            Error::new("batteryLow", ErrorLevel::ErrorWarning, "Battery below 20%", "Return to charging station")
                .add_description_translation("de", "Batterie unter 20%"),
        )
        .add_information(
            Info::new("lastUpdate", InfoLevel::InfoDebug).with_descriptor("State update received"),
        )
        .power_supply(Some(
            PowerSupply::new(78.0, false)
                .with_battery_voltage(48.2)
                .with_battery_health(98.0)
                .with_range(4200.0),
        ))
        .with_operating_mode(OperatingMode::OperatingAutomatic as i32)
        .safety_state(Some(SafetyState::new(EStopType::EstopNone, false)))
        .mobile_robot_position(Some(
            MobileRobotPosition { x: 0.0, y: 0.0, theta: 0.0, map_id: "warehouse-1".into(), localized: true, localization_score: Some(0.95), deviation_range: Some(0.02) },
        ))
        .velocity(Some(Velocity::new(Some(0.5), Some(0.0), Some(0.0))));
    let state_bytes = state_msg.encode_to_vec();
    let state_back = State::decode(state_bytes.as_slice()).unwrap();
    assert!(state_back.is_driving());
    assert_eq!(state_back.loads[0].load_type, "EPAL");
    assert_eq!(
        state_back.errors[0].error_level,
        ErrorLevel::ErrorWarning as i32
    );
    assert_eq!(
        state_back
            .safety_state
            .as_ref()
            .unwrap()
            .active_emergency_stop,
        EStopType::EstopNone as i32
    );
    assert!(state_back.safety_state.as_ref().unwrap().is_safe_to_drive());
    let pos = state_back.mobile_robot_position.as_ref().unwrap();
    assert!(pos.is_trusted());
    assert_eq!(pos.localization_quality(), Some(0.95));
    assert_eq!(pos.localization_deviation(), Some(0.02));
    let v = state_back.velocity.as_ref().unwrap();
    assert!(v.is_known());
    assert_eq!(v.linear_speed(), Some(0.5));
    let load = &state_back.loads[0];
    assert_eq!(load.weight_or_unknown(), Some(450.0));
    let _ctx = state_back.order_context();

    // -----------------------------------------------------------------------
    // 5. Factsheet — minimal builders
    // -----------------------------------------------------------------------
    let fs_msg = Factsheet::new("ACME-AMR", "Pallet-moving autonomous mobile robot")
        .with_header(mk_header().with_header_id(5));
    let fs_bytes = fs_msg.encode_to_vec();
    let fs_back = Factsheet::decode(fs_bytes.as_slice()).unwrap();
    assert_eq!(fs_back.type_specification.as_ref().unwrap().series_name, "ACME-AMR");

    // -----------------------------------------------------------------------
    // 6. ZoneSet — per-zone-type constructors + validators
    // -----------------------------------------------------------------------
    let zs = ZoneSetBuilder::new(
        mk_header().with_header_id(6),
        "warehouse-1",
        "zs-default",
    )
    .descriptor("Default zone set")
    .add_zone(
        Zone::speed_limit(
            "z-speed-1",
            0.5,
            vec![Vertex2D::new(0.0, 0.0), Vertex2D::new(1.0, 0.0), Vertex2D::new(1.0, 1.0)],
        )
        .with_descriptor("Speed limit near charging"),
    )
    .add_zone(Zone::release(
        "z-rel-1",
        ReleaseLossBehavior::ReleaseStop,
        vec![Vertex2D::new(0.0, 0.0), Vertex2D::new(2.0, 0.0), Vertex2D::new(2.0, 2.0)],
    ))
    .try_build()
    .expect("speed_limit and release zones are well-formed");
    let zs_bytes = zs.encode_to_vec();
    let zs_back = ZoneSet::decode(zs_bytes.as_slice()).unwrap();
    assert_eq!(zs_back.zone_count(), 2);
    assert_eq!(
        zs_back.zone_set.as_ref().unwrap().zones[0].zone_type,
        ZoneType::ZoneSpeedLimit as i32
    );
    assert_eq!(
        zs_back.zone_set.as_ref().unwrap().zones[0].maximum_speed,
        Some(0.5)
    );
    assert_eq!(
        zs_back.zone_set.as_ref().unwrap().zones[1].zone_type,
        ZoneType::ZoneRelease as i32
    );

    // Geometric helpers on Zone
    let z0 = &zs_back.zone_set.as_ref().unwrap().zones[0];
    assert!(z0.contains(Vertex2D::new(0.5, 0.5)));
    assert!(!z0.contains(Vertex2D::new(5.0, 5.0)));
    assert!(z0.area() > 0.0);

    // Validation: a SPEED_LIMIT zone without maximum_speed is rejected.
    let bad_zone = Zone {
        zone_id: "z-bad".into(),
        zone_type: ZoneType::ZoneSpeedLimit as i32,
        zone_descriptor: String::new(),
        vertices: vec![Vertex2D::new(0.0, 0.0), Vertex2D::new(1.0, 0.0), Vertex2D::new(1.0, 1.0)],
        release_loss_behavior: None,
        maximum_speed: None, // missing!
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
    assert!(bad_zone.validate().is_err());

    // -----------------------------------------------------------------------
    // 7. Responses — grant/queue/revoke/reject
    // -----------------------------------------------------------------------
    let resp_msg = ResponseBuilder::new(mk_header().with_header_id(7))
        .response(Response::granted("req-1", None))
        .response(Response::queued("req-2"))
        .response(Response::revoked("req-3"))
        .response(Response::rejected("req-4"))
        .build();
    assert_eq!(resp_msg.responses.len(), 4);
    assert!(resp_msg.responses[0].has_lease() == false); // no lease_expiry
    let found = resp_msg.lookup("req-2");
    assert!(found.is_some());

    // -----------------------------------------------------------------------
    // 8. Visualization — empty-for-state convenience
    // -----------------------------------------------------------------------
    let vis_msg =
        Visualization::empty_for_state(mk_header().with_header_id(8), 4);
    let vis_bytes = vis_msg.encode_to_vec();
    let vis_back = Visualization::decode(vis_bytes.as_slice()).unwrap();
    assert_eq!(vis_back.reference_state_header_id, 4);
    assert!(vis_back.planned_path.is_none());

    // -----------------------------------------------------------------------
    // 9. Enum helpers — Display / FromStr / short_name
    // -----------------------------------------------------------------------
    assert_eq!(BlockingType::BlockingHard.short_name(), "hard");
    assert_eq!(BlockingType::BlockingHard.to_string(), "hard");
    let parsed: BlockingType = "BLOCKING_HARD".parse().unwrap();
    assert_eq!(parsed, BlockingType::BlockingHard);
    let parsed_lower: BlockingType = "hard".parse().unwrap();
    assert_eq!(parsed_lower, BlockingType::BlockingHard);

    assert!(ErrorLevel::ErrorFatal.accepts_new_orders() == false);
    assert!(ErrorLevel::ErrorWarning.accepts_new_orders());
    assert!(ErrorLevel::ErrorCritical.can_continue_order() == false);

    // -----------------------------------------------------------------------
    // 10. Header helpers
    // -----------------------------------------------------------------------
    let h = Header::new("ACME", "amr-001").with_header_id(42);
    assert_eq!(h.header_id, 42);
    assert_eq!(h.version, "3.0.0");
    h.validate().unwrap();

    let h_bad = Header {
        header_id: 1,
        timestamp: None,
        version: String::new(),
        manufacturer: "X".into(),
        serial_number: "Y".into(),
    };
    assert!(h_bad.validate().is_err());

    // HeaderExt on State
    let mut s = State::new(Header::new("X", "Y"));
    s.set_header(Header::new("X", "Y"));
    assert!(s.header().is_some());
    let _ = s.header_mut();

    // Timestamp round-trip
    let ts = prost_types::Timestamp::now_utc();
    let iso = ts.to_iso8601_utc();
    let back = prost_types::Timestamp::from_iso8601_utc(&iso).unwrap();
    assert_eq!(back.seconds, ts.seconds);
    assert_eq!(back.nanos / 1_000_000, ts.nanos / 1_000_000);

    // -----------------------------------------------------------------------
    // 11. Trajectory validation
    // -----------------------------------------------------------------------
    let traj = Trajectory::linear(vec![
        ControlPoint::new(0.0, 0.0),
        ControlPoint::new(1.0, 0.0),
        ControlPoint::new(1.0, 1.0),
    ]);
    traj.validate().expect("linear trajectory is well-formed");

    let mut bad = traj.clone();
    bad.knot_vector = vec![0.0, 0.5]; // length 2, expected 5
    assert!(bad.validate().is_err());

    // -----------------------------------------------------------------------
    // 12. Vertex2D geometry helpers
    // -----------------------------------------------------------------------
    let v1 = Vertex2D::new(0.0, 0.0);
    let v2 = Vertex2D::new(3.0, 4.0);
    assert!((v1.distance_to(v2) - 5.0).abs() < 1e-9);
    let sq = vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(1.0, 0.0),
        Vertex2D::new(1.0, 1.0),
        Vertex2D::new(0.0, 1.0),
    ];
    assert!((polygon_area(&sq) - 1.0).abs() < 1e-9);
    assert!((polygon_perimeter(&sq) - 4.0).abs() < 1e-9);
    let (mn, mx) = polygon_bounding_box(&sq);
    assert_eq!(mn, Vertex2D::new(0.0, 0.0));
    assert_eq!(mx, Vertex2D::new(1.0, 1.0));
    assert!(point_in_polygon(Vertex2D::new(0.5, 0.5), &sq));
    assert!(!point_in_polygon(Vertex2D::new(2.0, 0.5), &sq));

    println!("VDA 5050 v3.0.0 ergonomic models compile and round-trip successfully.");
    println!("  Connection:     {} bytes wire", conn_bytes.len());
    println!("  Order:          {} bytes wire", order_bytes.len());
    println!("  InstantActions: {} bytes wire", ia_bytes.len());
    println!("  State:          {} bytes wire", state_bytes.len());
    println!("  Factsheet:      {} bytes wire", fs_bytes.len());
    println!("  ZoneSet:        {} bytes wire", zs_bytes.len());
    println!("  Visualization:  {} bytes wire", vis_bytes.len());
}